use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use rusqlite::Connection;
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

async fn insert_message(mp: &MessagePack, conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("INSERT INTO chat (name, message, date) VALUES (?, ?, ?)")?;
    stmt.execute(rusqlite::params![mp.name, mp.message, mp.date])?;
    Ok(())
}
fn get_all_messages(conn: &Connection, last_id: i64, count: u32) -> rusqlite::Result<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, message, date FROM chat WHERE id < ? ORDER BY id DESC LIMIT ?",
    )?;
    let message_iter = stmt.query_map(rusqlite::params![last_id, count], |row| {
        Ok(Message {
            id: row.get(0)?,
            name: row.get(1)?,
            message: row.get(2)?,
            date: row.get(3)?,
        })
    })?;

    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message?);
    }
    Ok(messages)
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let sql_conn = Arc::new(tokio::sync::Mutex::new(
        Connection::open("./data.db").unwrap(),
    ));
    let history_sql = sql_conn.clone();

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let history = warp::path("getHistory")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and_then(move |params: std::collections::HashMap<String, String>| {
            let sql_conn = history_sql.clone();
            async move {
                let latest_id: i64 = params.get("latestID").unwrap().parse().unwrap();
                let count: u32 = params.get("count").unwrap().parse().unwrap();
                let conn = sql_conn.lock().await;
                match get_all_messages(&conn, latest_id, count) {
                    Ok(messages) => Ok(warp::reply::json(&messages)),
                    Err(_) => Err(warp::reject::not_found()),
                }
            }
        });

    let (tx_broadcast, mut rx_broadcast) = broadcast::channel::<String>(100);


    let ws_route = warp::path("ws").and(warp::ws()).map(move |ws: warp::ws::Ws| {
        let mut rx_broadcast = tx_broadcast.subscribe();
        let tx_broadcast = tx_broadcast.clone();


        ws.on_upgrade( |websocket| async move {
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
                log::info!("发送消息: {}", text);
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
            let sql_conn_clone = sql_conn.clone();
            tokio::task::spawn_blocking(move || {
                let conn = futures::executor::block_on(sql_conn_clone.lock());
                if let Err(e) =
                    futures::executor::block_on(insert_message(&mp, &conn))
                {
                    log::error!("写入数据库失败: {}", e);
                } else {
                    log::debug!("写入数据库成功");
                }
            })
            .await
            .unwrap();
        }
    });


    let routes = hello.or(history).or(ws_route);
    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}
