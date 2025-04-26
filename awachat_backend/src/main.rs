use deadpool_sqlite::Manager;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use warp::Filter;

#[derive(Serialize)]
struct Message {
    id: i32,
    message: String,
    name: String,
    date: i64,
}
#[derive(Serialize, Deserialize)]
struct MessagePack {
    name: String,
    message: String,
    date: i64,
}

async fn insert_message(
    mp: MessagePack,
    conn: &deadpool::managed::Object<Manager>,
) -> Result<usize, deadpool_sqlite::rusqlite::Error>  {
    let res: Result<usize, deadpool_sqlite::rusqlite::Error> = conn
        .interact(move |conn| {
            conn.execute(
                "INSERT INTO chat (name, message, date) VALUES (?, ?, ?)",
                deadpool_sqlite::rusqlite::params![mp.name, mp.message, mp.date],
            )
        })
        .await
        .unwrap();
    res
}

async fn get_all_messages(
    conn: deadpool::managed::Object<Manager>,
    last_id: i64,
    count: u32,
) -> Result<Vec<Message>, ()> {
    let res=conn.interact(move |conn|{
        let mut stmt=conn.prepare("SELECT id, name, message, date FROM chat WHERE id < ? ORDER BY id DESC LIMIT ?").unwrap();
        let message_iter=stmt.query_map(deadpool_sqlite::rusqlite::params![last_id, count], |row| {
            Ok(Message {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                message: row.get(2).unwrap(),
                date: row.get(3).unwrap(),
            })
        }).unwrap();
        let messages=message_iter.filter_map(|msg| msg.ok()).collect::<Vec<Message>>();
        Ok(messages)
    }).await.unwrap();
    res
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cfg = deadpool_sqlite::Config::new("./data.db");
    let pool = cfg.create_pool(deadpool::Runtime::Tokio1).unwrap();
    let pool_clone = pool.clone();

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let history = warp::path("getHistory")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(warp::any().map(move || pool.clone()))
        .and_then(|params: std::collections::HashMap<String, String>, pool:deadpool::managed::Pool<Manager> | async move {
            let latest_id: i64 = params.get("latestID").unwrap().parse().unwrap();
            let count: u32 = params.get("count").unwrap().parse().unwrap();
            let conn = pool.get().await.unwrap();
            match get_all_messages(conn, latest_id, count).await {
                Ok(messages) => Ok(warp::reply::json(&messages)),
                Err(_) => Err(warp::reject::not_found()),
            }
        });

    let (tx_broadcast, mut rx_broadcast) = broadcast::channel::<String>(1024);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let mut rx_broadcast = tx_broadcast.subscribe();
            let tx_broadcast = tx_broadcast.clone();

            ws.on_upgrade(|websocket| async move {
                let (mut tx, mut rx) = websocket.split();

                tokio::spawn({
                    let tx_broadcast = tx_broadcast.clone();
                    async move {
                        while let Some(Ok(msg)) = rx.next().await {
                            if let Ok(text) = msg.to_str() {
                                let _ = tx_broadcast.send(text.to_string());
                                log::info!("好耶！！！收到消息了: {}", text);
                            }
                        }
                        log::info!("一个websocket连接关闭了");
                    }
                });

                while let Ok(text) = rx_broadcast.recv().await {
                    log::info!("广播消息: {}", text);
                    if tx.send(warp::ws::Message::text(text)).await.is_err() {
                        log::info!("无法发送 应该是关闭了吧QwQ 那我也走咯www");
                        break;
                    }
                }
            })
        });
    tokio::spawn(async move {
        while let Ok(text) = rx_broadcast.recv().await {
            log::debug!("准备写入数据库: {}", text);
            let mp: MessagePack = match serde_json::from_str(&text) {
                Ok(val) => val,
                Err(e) => {
                    log::error!("json解析错误: {}", e);
                    continue;
                }
            };
            let conn=pool_clone.get().await.unwrap();
            if let Err(e) = insert_message(mp, &conn).await {
                log::error!("写入数据库失败: {}", e);
            } else {
                log::debug!("写入数据库成功");
            }
        }
    });

    let routes = hello.or(history).or(ws_route);
    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}
