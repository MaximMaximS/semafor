use super::{
    config::{Light, Mode},
    util::AppError,
    AppState,
};
use crate::CONFIG;
use anyhow::Context;
use axum::extract::State;
use chrono::{DateTime, Duration};
use chrono_tz::{Europe::Prague, Tz};
use rezvrh_scraper::{Bakalari, Type};
use std::sync::Arc;
use timetabler::Timetabler;
use tracing::debug;

mod timetabler;

#[derive(Debug)]
pub struct BakaWrapper {
    timetabler: Timetabler,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LightState {
    Empty,
    Break,
    BeforeLesson,
    Lesson,
    BeforeBreak,
}

impl LightState {
    pub const fn light(self) -> Light {
        match self {
            Self::Empty => Light::Off,
            Self::Break => Light::Green,
            Self::BeforeLesson => Light::Amber,
            Self::Lesson => Light::Red,
            Self::BeforeBreak => Light::RedAmber,
        }
    }
}

impl BakaWrapper {
    pub async fn new(bakalari: Bakalari) -> anyhow::Result<Self> {
        let sel = bakalari
            .get_selector(Type::Room, &CONFIG.bakalari.room)
            .context("room not found")?;

        Ok(Self {
            timetabler: Timetabler::new(bakalari, sel).await?,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_state(&self) -> anyhow::Result<LightState> {
        let now = chrono::Local::now().with_timezone(&Prague);
        self.get_state_at(now).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_state_at(&self, date: DateTime<Tz>) -> anyhow::Result<LightState> {
        let table = self.timetabler.get_timetable().await?;

        let time_now = date.time();
        let date_now = date.date_naive();
        let day = table.days.iter().find(|d| d.date == Some(date_now));
        let Some(day) = day else {
            println!("No day found for {date_now:?}");
            return Ok(LightState::Empty);
        };
        let hours = &table.hours;
        assert!(hours.len() == day.lessons.len());
        let mut lessons = hours
            .iter()
            .zip(day.lessons.iter())
            .filter(|(_, l)| {
                if l.is_empty() {
                    return false;
                }
                let l = &l[0];
                matches!(
                    l,
                    rezvrh_scraper::Lesson::Regular { .. }
                        | rezvrh_scraper::Lesson::Substitution { .. }
                )
            })
            .peekable();
        let first = lessons.peek();
        debug!(first = ?first, "First lesson");
        let Some(first) = first else {
            debug!("No lessons");
            return Ok(LightState::Empty);
        };
        let tz_start = date.with_time(first.0.start).unwrap();
        debug!(tz_start = ?tz_start, "First lesson start");
        let lights_on = tz_start - CONFIG.time.lights_on;
        if date < lights_on {
            debug!("Too early for lights on");
            return Ok(LightState::Empty);
        }
        while let Some((hour, _)) = lessons.next() {
            let start = hour.start;
            let early_start = start - CONFIG.time.before_lesson;

            if time_now < early_start {
                return Ok(LightState::Break);
            }

            if time_now < start {
                return Ok(LightState::BeforeLesson);
            }

            let end = hour.start + Duration::minutes(hour.duration.into());
            let early_end = end - CONFIG.time.before_break;
            if time_now < early_end {
                return Ok(LightState::Lesson);
            }

            if time_now < end {
                return Ok(LightState::BeforeBreak);
            }

            if lessons.peek().is_some() {
                continue;
            }

            let lights_off = end + CONFIG.time.lights_off;

            if time_now < lights_off {
                return Ok(LightState::Break);
            }

            return Ok(LightState::Empty);
        }
        unreachable!("lessons.next() should have returned None");
    }
}

pub async fn get_light(State(state): State<Arc<AppState>>) -> Result<String, AppError> {
    let config = state.config.lock().await;
    let static_val = config.custom.to_val();
    let mode = config.mode;
    drop(config);
    let light = match mode {
        Mode::Static => static_val,
        Mode::Bakalari => state.bakalari.get_state().await?.light().to_val(),
    };
    Ok(format!("{}", light | 0b1000))
}
