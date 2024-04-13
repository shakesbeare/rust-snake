use std::sync::{Arc, Mutex, OnceLock};

use bevy::prelude::*;
use bevy::utils::tracing;
use futures::FutureExt;

pub static HIGHSCORES: OnceLock<Arc<Mutex<Highscores>>> = OnceLock::new();

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum ScoresDownloaded {
    #[default]
    NotDownloaded,
    Downloaded,
}

#[derive(Debug, Clone, Default, Event)]
pub struct AcquireHighscores;

#[derive(Debug, Clone, Default, Event)]
pub struct TriggerDownload;

#[derive(Debug, Clone, Event)]
pub struct SendHighscores(pub Highscore);

#[derive(Debug, Resource)]
pub enum LeaderboardEarned {
    Placed(u8),
    NotPlaced,
}

#[derive(Resource, Clone, Default)]
pub struct Score(pub u32);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Highscore {
    pub name: String,
    pub score: u32,
}

#[derive(
    Resource, Clone, Debug, serde::Serialize, serde::Deserialize, Default,
)]
pub struct Highscores {
    pub highscores: Vec<Highscore>,
}

pub fn download_manager(
    mut scores_downloaded: ResMut<NextState<ScoresDownloaded>>,
    mut acquire_highscores: EventReader<AcquireHighscores>,
    mut trigger_download: EventWriter<TriggerDownload>,
) {
    if acquire_highscores.read().next().is_some() {
        scores_downloaded.set(ScoresDownloaded::NotDownloaded);
        trigger_download.send(TriggerDownload);
    }
}

pub fn init_scores() {
    HIGHSCORES.get_or_init(|| Arc::new(Mutex::new(Highscores::default())));
}

pub fn download_scores(
    mut trigger_download: EventReader<TriggerDownload>,
) {
    
    if trigger_download.read().next().is_some() {
        crate::run_async(async move {
            let mut fut = tokio::task::spawn_local(async {
                let client = reqwest::Client::new();
                let res = client
                    .get("https://berintmoffett.com/api/snake-highscores")
                    .send()
                    .await
                    .unwrap();
                let text = res.text().await.unwrap();
                let highscores = serde_json::from_str::<Highscores>(&text);
                match highscores {
                    Ok(v) => v,
                    Err(e) => panic!("Error: {:#?}", e),
                }
            })
                .fuse();

            futures::select! {
                res = fut => {
                    let hs_arc = HIGHSCORES.get();
                    let mut highscores = hs_arc.unwrap().lock().unwrap();
                    *highscores = res.unwrap();
                }
            }
        });
    }

}

pub fn upload_scores(mut send_highscores: EventReader<SendHighscores>) {
    if let Some(ev) = send_highscores.read().next() {
        let highscore = ev.0.clone();
        debug!("{:?}", serde_json::to_string(&highscore).unwrap());
        crate::run_async(async move {
            let mut fut = tokio::task::spawn_local(async move {
                let client = reqwest::Client::new();
                let _ = client
                    .post("https://berintmoffett.com/api/snake-highscores")
                    .json(&highscore)
                    .send()
                    .await
                    .unwrap();
            })
            .fuse();

            futures::select! {
                _ = fut => tracing::debug!("Highscores uploaded"),
            }
        });
    }
}

#[derive(Component)]
pub struct ScoreText;

