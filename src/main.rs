use futures::TryStreamExt;
use mongodb::Client;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{thread, time};

#[derive(Debug, Serialize, Deserialize)]
struct MbTcpData {
    mb_lock_to_uid: String,
    mb_ip: String,
    mb_port: String,
    mb_register: String,
    mb_rw: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MbWriteVal {
    node_uid: String,
    node_val: String,
}

async fn loop_through_data(mb_datas: Vec<MbTcpData>) {
    for mb_data in mb_datas {
        // Handle modbus reads. Or other words reads out the modbus register, and stores the data in the database
        if mb_data.mb_rw == "r" {
            let read_data = data_concentrator_mb::read_mb(
                mb_data.mb_ip,
                mb_data.mb_port,
                mb_data.mb_register.parse().unwrap(),
                1,
            )
            .await
            .unwrap();

            let first_element = read_data[0];

            let mut map = HashMap::new();
            map.insert("node_val", first_element.to_string());
            map.insert("node_uid", mb_data.mb_lock_to_uid);

            let rw_client = reqwest::Client::new();

            let _res = rw_client
                .post("http://127.0.0.1:8000/w")
                .json(&map)
                .send()
                .await
                .unwrap();
        } else if mb_data.mb_rw == "w" {
            // Handle modbus writes. Or in other words, reads out the value from the data base, and writes it to the given register.
            let mut get_url = "http://127.0.0.1:8000/r/".to_string();
            get_url.push_str(&mb_data.mb_lock_to_uid);

            let resp = reqwest::get(get_url)
                .await
                .unwrap()
                .json::<MbWriteVal>()
                .await
                .unwrap();
            data_concentrator_mb::write_mb(
                mb_data.mb_ip,
                mb_data.mb_port,
                mb_data.mb_register.parse().unwrap(),
                resp.node_val,
            )
            .await
            .unwrap();
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Mongo client
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    // Mongo db
    let db = client.database("dconc");
    // Mongo collection
    let collection = db.collection::<MbTcpData>("mbstuff");
    // To sleep
    let ten_millis = time::Duration::from_secs(10);

    loop {
        let cursor = collection
            .find(None, None)
            .await
            .ok()
            .expect("Can't find data in coll");

        let mb_datas: Vec<MbTcpData> = cursor.try_collect().await.unwrap();
        // Loop thrugh the read out data.
        loop_through_data(mb_datas).await;

        thread::sleep(ten_millis);
    }
}
