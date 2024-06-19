use std::time::{Duration, Instant};

use rezvrh_scraper::{Bakalari, Selector, Timetable};
use tokio::sync::{mpsc, oneshot};
use tracing::info;

type TokenRequest = oneshot::Sender<Result<Timetable, rezvrh_scraper::Error>>;

#[derive(Debug)]
pub struct Timetabler {
    sender: mpsc::Sender<TokenRequest>,
}

async fn renew(
    bakalari: &Bakalari,
    selector: &Selector,
) -> Result<(Instant, Timetable), rezvrh_scraper::Error> {
    info!("Fetching timetable...");
    let table = bakalari
        .get_timetable(rezvrh_scraper::Which::Actual, selector)
        .await;

    table.map(|t| (Instant::now(), t))
}

impl Timetabler {
    pub async fn new(
        bakalari: Bakalari,
        selector: Selector,
    ) -> Result<Self, rezvrh_scraper::Error> {
        let (sender, mut receiver) = mpsc::channel::<TokenRequest>(32);
        let table = renew(&bakalari, &selector).await?;

        tokio::spawn(async move {
            let mut store = table;

            // Start receiving messages
            while let Some(sender) = receiver.recv().await {
                if store.0.elapsed() > Duration::from_secs(300) {
                    let new = renew(&bakalari, &selector).await;
                    match new {
                        Ok(table) => {
                            store = table;
                            continue;
                        }
                        Err(err) => {
                            sender.send(Err(err)).unwrap();
                            continue;
                        }
                    }
                }
                sender.send(Ok(store.1.clone())).unwrap();
            }
        });

        Ok(Self { sender })
    }

    pub async fn get_timetable(&self) -> Result<Timetable, rezvrh_scraper::Error> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(tx)
            .await
            .expect("failed to send token request");
        rx.await.unwrap()
    }
}
