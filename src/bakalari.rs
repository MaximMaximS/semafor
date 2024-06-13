use chrono::{DateTime, Duration};
use chrono_tz::{Europe::Prague, Tz};
use rezvrh_scraper::{Bakalari, Selector, Timetable, Type};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Options {
    /// Seconds
    pub lights_on: u32,
    /// Seconds
    pub lights_off: u32,
    /// Seconds
    pub before_break: u32,
    /// Seconds
    pub before_lesson: u32,
}

#[derive(Debug)]
pub struct BakaWrapper {
    bakalari: Bakalari,
    selector: Selector,
    options: Options,
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
    pub fn new(bakalari: Bakalari, room: &str, options: Options) -> Option<Self> {
        let sel = bakalari.get_selector(Type::Room, room);

        Some(Self {
            bakalari,
            selector: sel?,
            timetable: Mutex::new(None),
            options,
        })
    }

    pub async fn get_state(&self) -> anyhow::Result<State> {
        let now = chrono::Local::now().with_timezone(&Prague);
        self.get_state_at(now).await
    }

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
        let Some(first) = first else {
            return Ok(State::Empty);
        };
        let tz_start = date.with_time(first.0.start).unwrap();
        let lights_on = tz_start - Duration::seconds(i64::from(self.options.lights_on));
        if date < lights_on {
            return Ok(State::Empty);
        }
        while let Some((hour, _)) = lessons.next() {
            let start = hour.start;
            let early_start = start - Duration::seconds(i64::from(self.options.before_lesson));

            if time_now < early_start {
                return Ok(State::Break);
            }

            if time_now < start {
                return Ok(State::BeforeLesson);
            }

            let end = hour.start + Duration::minutes(hour.duration.into());
            let early_end = end - Duration::seconds(i64::from(self.options.before_break));
            if time_now < early_end {
                return Ok(State::Lesson);
            }

            if time_now < end {
                return Ok(State::BeforeBreak);
            }

            if lessons.peek().is_some() {
                continue;
            }

            let lights_off = end + Duration::seconds(i64::from(self.options.lights_off));

            if time_now < lights_off {
                return Ok(State::Break);
            }

            return Ok(State::Empty);
        }
        unreachable!("lessons.next() should have returned None");
    }
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;

    #[tokio::test]
    async fn test_get_state() {
        let baka = Bakalari::no_auth("https://gymberoun.bakalari.cz".parse().unwrap())
            .await
            .unwrap();
        let wrapper = BakaWrapper::new(
            baka,
            "216",
            Options {
                lights_on: 30 * 60,
                lights_off: 10 * 60,
                before_break: 60,
                before_lesson: 60,
            },
        )
        .unwrap();

        let mut date = chrono::Local::now()
            .with_timezone(&Prague)
            .with_hour(6)
            .unwrap();
        for _ in 0..1000 {
            let state = wrapper.get_state_at(date).await.unwrap();
            println!("{date:?} {state:?}");
            date += Duration::minutes(1);
        }
    }
}
