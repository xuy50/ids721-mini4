use axum::{Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, GetObjectRequest};
use csv::ReaderBuilder;
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Product")]
    product: String,
    #[serde(rename = "Price")]
    price: f64,
    #[serde(rename = "Quantity")]
    quantity: i32,
}

pub async fn price_filter(axum::extract::Path((low, high)): axum::extract::Path<(f64, f64)>) -> Result<Json<Vec<Record>>, StatusCode> {
    let s3_client = S3Client::new(Region::UsEast2);
    let get_req = GetObjectRequest {
        bucket: "ids-721-data".to_string(),
        key: "dataset_sample.csv".to_string(),
        ..Default::default()
    };

    let result = s3_client.get_object(get_req).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = result.body.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut body_reader = body.into_async_read();
    let mut csv_content = Vec::new();
    body_reader.read_to_end(&mut csv_content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rdr = ReaderBuilder::new().from_reader(csv_content.as_slice());
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if record.price >= low && record.price <= high {
            records.push(record);
        }
    }

    Ok(Json(records))
    // let response_body = serde_json::to_string(&records).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Ok(Json(response_body))
}

//test
#[cfg(test)]
mod tests {
    use super::*;

    fn filter_records(records: Vec<Record>, low_price: f64, high_price: f64) -> Vec<Record> {
        records.into_iter()
               .filter(|r| r.price >= low_price && r.price <= high_price)
               .collect()
    }

    #[tokio::test]
    async fn test_price_filter() {
        let records = vec![
            Record { date: "2023-09-01".into(), product: "Apple".into(), price: 1.2, quantity: 50 },
            Record { date: "2023-09-01".into(), product: "Banana".into(), price: 0.5, quantity: 40 },
            Record { date: "2023-09-01".into(), product: "Cherry".into(), price: 2.5, quantity: 20 },
        ];

        let filtered_records = filter_records(records, 1.0, 2.0);

        assert_eq!(filtered_records.len(), 1);
        assert_eq!(filtered_records[0].product, "Apple");
    }
}
