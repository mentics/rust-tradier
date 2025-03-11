use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use serde_json::Value;
use std::result::Result;

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    // println!("#### deserializing ####");
    let x = Value::deserialize(deserializer).map(|doc| match doc {
        Value::String(s) => {
            // println!("s: {:?}", s);
            DateTime::parse_from_rfc3339(&s)
                .unwrap()
                .with_timezone(&Utc)
        }
        Value::Object(o) => {
            // println!("doc: {:?}", o);
            let date = o.get("$date").expect("expected $date");
            // println!("date: {:?}", date);
            let num = date.get("$numberLong").expect("expected $numberLong");
            // println!("num: {:?}", num);
            let millis = match num {
                Value::String(s) => s.parse::<i64>().unwrap(),
                Value::Number(n) => n.as_i64().unwrap(),
                _ => panic!("expected string or number"),
            };
            DateTime::from_utc(NaiveDateTime::from_timestamp(millis / 1000, 0), Utc)
        }
        _ => panic!("expected string or object for datetime"),
    })?;
    Ok(x)
}

pub fn serialize<S>(source: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let millis = source.timestamp_millis();
    let json_value = json!({ "$date": { "$numberLong": millis.to_string() } });
    json_value.serialize(serializer)
    // let mut map = serde_json::Map::new();
    // map.insert(
    //     "$date".to_string(),
    //     Value::Number(serde_json::Number::from(millis)),
    // );
    // let value = Value::Object(map);
    // value.serialize(serializer)
}

// serializer.wr source.timestamp_millis()
// DateTime::from(*source).serialize(serializer)

// /// Deserializes a [`chrono::DateTime`] from a [`crate::DateTime`].
// #[cfg_attr(docsrs, doc(cfg(feature = "chrono-0_4")))]
// pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::DateTime<Utc>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let datetime = DateTime::deserialize(deserializer)?;
//     Ok(datetime.to_chrono())
// }

// /// Serializes a [`chrono::DateTime`] as a [`crate::DateTime`].
// #[cfg_attr(docsrs, doc(cfg(feature = "chrono-0_4")))]
// pub fn serialize<S: Serializer>(
//     val: &chrono::DateTime<Utc>,
//     serializer: S,
// ) -> Result<S::Ok, S::Error> {
//     let datetime = DateTime::from_chrono(val.to_owned());
//     datetime.serialize(serializer)
// }

// fn main() {
//     let bson_doc = bson::doc! {
//         "transaction_date": {
//             "$date": {
//                 "$numberLong": "1741019434527"
//             }
//         }
//     };

//     let transaction: Transaction = bson::from_bson(Bson::Document(bson_doc)).unwrap();
//     println!("{:?}", transaction.transaction_date);

//     let bson_doc2 = bson::to_bson(&transaction).unwrap().as_document().unwrap().clone();
//     let transaction2: Transaction = bson::from_bson(Bson::Bson::Document(bson_doc2)).unwrap();
//     println!("{:?}", transaction2.transaction_date);
// }
