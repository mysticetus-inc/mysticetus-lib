use firestore_rs::Firestore;
use rand::Rng;
use rand::rngs::ThreadRng;

async fn get_client() -> Firestore {
    Firestore::new("mysticetus-oncloud", gcp_auth_channel::Scope::Firestore)
        .await
        .expect("should be able to build client")
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TestDoc {
    id: uuid::Uuid,
    string: String,
    number: i32,
    float: f64,
    timestamp: timestamp::Timestamp,
    geo: firestore_rs::LatLng,
    boolean: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reference: Option<firestore_rs::Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nested: Option<Box<TestDoc>>,
}

impl TestDoc {
    fn random() -> Self {
        const DEFAULT_LAYERS: usize = 3;
        Self::random_with_layers(DEFAULT_LAYERS)
    }

    fn random_with_layers(layers: usize) -> Self {
        // rand itself generates this from a cached thread local, so this is a relatively cheap
        // function (after the first call, that is)
        let mut rng = rand::rng();

        Self::build_nested(&mut rng, layers)
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }

    fn build_nested(rng: &mut ThreadRng, nested_layers: usize) -> Self {
        fn gen_string(rng: &mut ThreadRng) -> String {
            let len = rng.random_range(1_usize..=50);

            let mut dst = String::with_capacity(len);

            for _ in 0..len {
                let mut b = rng.random_range(b'0'..=b'z');
                while !b.is_ascii_alphanumeric() {
                    b = rng.random_range(b'0'..=b'z');
                }

                dst.push(b as char);
            }

            dst
        }

        fn gen_timestamp(rng: &mut ThreadRng) -> timestamp::Timestamp {
            thread_local!(static NOW: f64 = timestamp::Timestamp::now().as_seconds_f64());
            const START: f64 = timestamp::Timestamp::UNIX_EPOCH.as_seconds_f64();

            let now = NOW.with(|ts| *ts);

            let ts = rng.random_range(START..=now);

            timestamp::Timestamp::from_seconds_f64_checked(ts).unwrap()
        }

        fn gen_geo(rng: &mut ThreadRng) -> firestore_rs::LatLng {
            let latitude = rng.random_range(-90.0..=90.0);
            let longitude = rng.random_range(-180.0..=180.0);

            firestore_rs::LatLng {
                latitude,
                longitude,
            }
        }

        let mut new = Self {
            id: uuid::Uuid::nil(),
            number: rng.random(),
            string: gen_string(rng),
            float: rng.random(),
            timestamp: gen_timestamp(rng),
            geo: gen_geo(rng),
            boolean: rng.random(),
            nested: None,
            reference: None,
        };

        if nested_layers > 0 {
            new.nested = Some(Box::new(Self::build_nested(rng, nested_layers - 1)));
        }

        new
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_collection() -> firestore_rs::Result<()> {
    let client = get_client().await;

    let count = client.collection("tests").delete().await?;
    println!("deleted {count} docs");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_set_get_delete() -> firestore_rs::Result<()> {
    let client = get_client().await;

    let test_doc = TestDoc::random();

    let json_str = serde_json::to_string_pretty(&test_doc).unwrap();
    println!("json_repr: {}", json_str);

    client
        .collection("tests")
        .doc("test_set_get_delete")
        .update(&test_doc)
        .await?;

    let retrieved_doc: firestore_rs::Doc<TestDoc> = client
        .collection("tests")
        .doc("test_set_get_delete")
        .get()
        .await?
        .expect("we just set the document, it should exist");

    assert_eq!(test_doc, retrieved_doc.into_inner());

    client
        .collection("tests")
        .doc("test_set_get_delete")
        .delete()
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_transform() -> firestore_rs::Result<()> {
    let client = get_client().await;

    let doc = serde_json::json!({
        "a": "value",
        "b": "value2",
    });

    client
        .collection("tests")
        .doc("transforms")
        .build_write()
        .update(&doc)?
        .field_increment("incr", 5)
        .commit()
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_update() -> firestore_rs::Result<()> {
    use serde_json::json;

    let client = get_client().await;

    let doc_ref = client.collection("tests").doc("test_update");

    let initial = json!({
        "a": "original",
        "b": "deleted",
        "const": "constant",
        "nested": {
            "c": "original",
            "d": ["original"],
            "e": "deleted",
            "const": "constant",
        }
    });

    let update = json!({
        "a": "updated",
        "b": null,
        "nested": {
            "c": "updated",
            "d": ["updated"],
            "e": null,
        }
    });

    let final_expected = json!({
        "a": "updated",
        "const": "constant",
        "b": null,
        "nested": {
            "c": "updated",
            "d": ["updated"],
            "const": "constant",
            "e": null,
        }
    });

    doc_ref.set(&initial).await?;

    let final_resp: serde_json::Value = doc_ref.update(&update).await?.into_inner();

    println!("final: {:#?}", final_resp);
    println!("final_expected: {:#?}", final_expected);

    assert_eq!(final_resp, final_expected);

    Ok(())
}

#[tokio::test]
async fn test_get_field() -> firestore_rs::Result<()> {
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_batch_write() -> firestore_rs::Result<()> {
    let client = get_client().await;

    let mut batch_write = client.batch_write();

    batch_write
        .collection("test-1")
        .doc("test-a")
        .set(&TestDoc::random())?;
    batch_write
        .collection("test-2")
        .doc("test-b")
        .set(&TestDoc::random())?;

    batch_write.commit().await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn multiple_clients() -> firestore_rs::Result<()> {
    use futures::StreamExt;

    let client = get_client().await;

    let docs = std::iter::repeat_with(TestDoc::random).take(10);

    let base_collec_ref = client.collection("tests");

    let mut handles = futures::stream::FuturesUnordered::new();

    for (idx, doc) in docs.into_iter().enumerate() {
        let collec_ref = base_collec_ref.clone();

        handles.push(tokio::spawn(async move {
            collec_ref.doc(doc.id()).set(&doc).await?;

            println!("sent doc number {idx}");

            Ok(()) as firestore_rs::Result<()>
        }));
    }

    while let Some(result) = handles.next().await {
        result.expect("could not join handle").unwrap();
    }

    Ok(())
}

#[tokio::test]
async fn test_paths() -> firestore_rs::Result<()> {
    use serde_json::json;

    let client = get_client().await;

    let doc_json = json!({
        "ok_path": "value",
        "path with space": "value",
        "path'with'ticks": "value",
        "path with'both": "value",
        "nested object": {
            "ok_path": "value",
            "path with space": "value",
            "path'with'ticks": "value",
            "path with'both": "value",
        }
    });

    let returned: serde_json::Value = client
        .collection("tests")
        .doc("test_paths")
        .set(&doc_json)
        .await?
        .into_inner();

    assert_eq!(doc_json, returned);

    Ok(())
}

#[tokio::test]
async fn test_get_path() -> firestore_rs::Result<()> {
    use std::collections::HashMap;

    let client = get_client().await;

    let returned: HashMap<String, serde_json::Value> = client
        .collection("tests")
        .doc("get")
        .get()
        .await?
        .unwrap()
        .into_inner();

    println!("{:#?}", returned);

    Ok(())
}

#[ignore = "needs to be refactored to use the 'mysticetus-oncloud' project"]
#[tokio::test(flavor = "multi_thread")]
async fn test_query() -> firestore_rs::Result<()> {
    use std::collections::HashMap;

    use futures::StreamExt;

    let client =
        firestore_rs::Firestore::new("winged-citron-305220", gcp_auth_channel::Scope::Firestore)
            .await?;

    let mut result_stream = client
        .collection("videos")
        .query()
        .where_field("cameraType")
        .equals("infrared")
        .limit(5)
        .run()
        .await?;

    let mut results = Vec::new();

    while let Some(next_result) = result_stream.next().await {
        let doc_opt: Option<HashMap<String, serde_json::Value>> = next_result?;

        if let Some(doc) = doc_opt {
            results.push(doc);
        }
    }

    let target_value = serde_json::Value::String("infrared".into());

    for doc in results.iter() {
        assert_eq!(doc.get("cameraType"), Some(&target_value));
    }

    println!("found {} docs", results.len());

    if let Some(first) = results.first() {
        println!("first doc: {:#?}", first);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_query_by_name() -> firestore_rs::Result<()> {
    let client = get_client().await;

    let test_doc = TestDoc::random();

    client
        .collection("tests")
        .doc(test_doc.id())
        .set(&test_doc)
        .await?;

    let result: TestDoc = client
        .collection("tests")
        .query()
        .where_id()
        .equals(test_doc.id())
        .first()
        .await?
        .expect("we just set the doc, it should exist");

    assert_eq!(result, test_doc);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_list_collection_ids() -> firestore_rs::Result<()> {
    use futures::StreamExt;

    let mut client = get_client().await.clone();

    let mut id_stream = Box::pin(client.list_collection_ids());
    let mut ids = Vec::new();

    while let Some(result) = id_stream.next().await {
        let new_batch = result?;

        println!("new batch: {:?}", new_batch);

        ids.extend(new_batch);
    }

    assert!(!ids.is_empty());
    assert!(ids.contains(&String::from("tests")));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_subcollection() -> firestore_rs::Result<()> {
    let client = get_client().await;

    let subcollection_doc = TestDoc::random();
    let test_doc = TestDoc::random();

    // make sure the parent document exists
    client
        .collection("tests")
        .doc(subcollection_doc.id())
        .set(&subcollection_doc)
        .await?;

    // Create the subcollection and insert the test document
    client
        .collection("tests")
        .doc(subcollection_doc.id())
        .collection("subcollection")
        .doc(test_doc.id())
        .set(&test_doc)
        .await?;

    // retrieve the document we just set.
    let retrieved_test_doc: TestDoc = client
        .collection("tests")
        .doc(subcollection_doc.id())
        .collection("subcollection")
        .doc(test_doc.id())
        .get()
        .await?
        .expect("document we just set is missing")
        .into_inner();

    // Verify it's exactly the same.
    assert_eq!(test_doc, retrieved_test_doc);

    Ok(())
}
