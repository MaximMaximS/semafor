use chrono::{DateTime, Duration};
use chrono_tz::{Europe::Prague, Tz};
use rezvrh_scraper::{Bakalari, Selector, Timetable, Type};
use tokio::sync::Mutex;
use tracing::debug;

use crate::CONFIG;

#[derive(Debug)]
pub struct BakaWrapper {
    bakalari: Bakalari,
    selector: Selector,
    timetable: Mutex<Option<Timetable>>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum State {
    Empty,
    Break,
    BeforeLesson,
    Lesson,
    BeforeBreak,
}

impl State {
    pub const fn light(self) -> u8 {
        match self {
            Self::Empty => 0b000,
            Self::Break => 0b100,
            Self::BeforeLesson => 0b010,
            Self::Lesson => 0b001,
            Self::BeforeBreak => 0b011,
        }
    }
}

impl BakaWrapper {
    pub fn new(bakalari: Bakalari) -> Option<Self> {
        let sel = bakalari.get_selector(Type::Room, &CONFIG.bakalari.room);

        Some(Self {
            bakalari,
            selector: sel?,
            timetable: Mutex::new(None),
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_state(&self) -> anyhow::Result<State> {
        let now = chrono::Local::now().with_timezone(&Prague);
        self.get_state_at(now).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_state_at(&self, date: DateTime<Tz>) -> anyhow::Result<State> {
        let mut table_l = self.timetable.lock().await;
        if table_l.is_none() {
            *table_l = Some(
                self.bakalari
                    .get_timetable(rezvrh_scraper::Which::Actual, &self.selector)
                    .await?,
            );
        }

        let table = table_l.as_ref().unwrap().clone();
        drop(table_l);
        debug!(table = ?table, "Got timetable");

        let time_now = date.time();
        let date_now = date.date_naive();
        let day = table.days.iter().find(|d| d.date == Some(date_now));
        let Some(day) = day else {
            println!("No day found for {date_now:?}");
            return Ok(State::Empty);
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
            return Ok(State::Empty);
        };
        let tz_start = date.with_time(first.0.start).unwrap();
        debug!(tz_start = ?tz_start, "First lesson start");
        let lights_on = tz_start - CONFIG.time.lights_on;
        if date < lights_on {
            debug!("Too early for lights on");
            return Ok(State::Empty);
        }
        while let Some((hour, _)) = lessons.next() {
            let start = hour.start;
            let early_start = start - CONFIG.time.before_lesson;

            if time_now < early_start {
                return Ok(State::Break);
            }

            if time_now < start {
                return Ok(State::BeforeLesson);
            }

            let end = hour.start + Duration::minutes(hour.duration.into());
            let early_end = end - CONFIG.time.before_break;
            if time_now < early_end {
                return Ok(State::Lesson);
            }

            if time_now < end {
                return Ok(State::BeforeBreak);
            }

            if lessons.peek().is_some() {
                continue;
            }

            let lights_off = end + CONFIG.time.lights_off;

            if time_now < lights_off {
                return Ok(State::Break);
            }

            return Ok(State::Empty);
        }
        unreachable!("lessons.next() should have returned None");
    }
}
