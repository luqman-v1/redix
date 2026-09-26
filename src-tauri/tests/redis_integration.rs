// Pongolong: Tests use #[ignore] — run with `cargo test -- --ignored` when Redis is up
use redix_app_lib::config::ConnectionConfig;
use redix_app_lib::redis::{RedisClient, RedisValue, StandaloneClient};

fn test_config() -> ConnectionConfig {
    ConnectionConfig::new("test", "127.0.0.1", 6399)
}

#[tokio::test]
#[ignore]
async fn test_connect_and_ping() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();
    let result = client.ping().await.unwrap();
    assert!(result);
}

#[tokio::test]
#[ignore]
async fn test_set_get_del() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();

    // SET
    let result = client
        .execute("SET", vec!["test_key".into(), "hello".into()])
        .await
        .unwrap();
    assert_eq!(result, RedisValue::Status("OK".to_string()));

    // GET
    let result = client
        .execute("GET", vec!["test_key".into()])
        .await
        .unwrap();
    assert_eq!(result, RedisValue::String("hello".to_string()));

    // DEL
    let result = client
        .execute("DEL", vec!["test_key".into()])
        .await
        .unwrap();
    assert_eq!(result, RedisValue::Integer(1));

    // GET after DEL → nil
    let result = client
        .execute("GET", vec!["test_key".into()])
        .await
        .unwrap();
    assert!(result.is_nil());
}

#[tokio::test]
#[ignore]
async fn test_scan_keys() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();

    // Seed some keys
    for i in 0..3 {
        client
            .execute("SET", vec![format!("scan_test:{}", i), format!("val{}", i)])
            .await
            .unwrap();
    }

    // SCAN for them
    let (_, keys) = client.scan_keys(0, 100, Some("scan_test:*")).await.unwrap();
    assert!(keys.iter().any(|k| k.contains("scan_test:")));

    // Cleanup
    for i in 0..3 {
        client
            .execute("DEL", vec![format!("scan_test:{}", i)])
            .await
            .unwrap();
    }
}

#[tokio::test]
#[ignore]
async fn test_ttl_pipeline_matches_per_key_ttl() {
    let mut client = StandaloneClient::new(test_config());
    client.connect().await.unwrap();

    // Seed a mix of expiring and persistent keys.
    client
        .execute("SET", vec!["pipe:persistent".into(), "a".into()])
        .await
        .unwrap();
    client
        .execute("SET", vec!["pipe:expiring".into(), "b".into()])
        .await
        .unwrap();
    client
        .execute("EXPIRE", vec!["pipe:expiring".into(), "120".into()])
        .await
        .unwrap();

    let keys = vec![
        "pipe:persistent".to_string(),
        "pipe:expiring".to_string(),
        "pipe:missing".to_string(),
    ];

    // This is the shape scan_keys uses to replace one TTL call per key.
    let cmds: Vec<(String, Vec<String>)> = keys
        .iter()
        .map(|k| ("TTL".to_string(), vec![k.clone()]))
        .collect();
    let pipelined = client.execute_pipeline(cmds).await.unwrap();

    assert_eq!(pipelined.len(), keys.len());
    for (key, value) in keys.iter().zip(pipelined.iter()) {
        let from_pipeline = match value {
            RedisValue::Integer(n) => *n,
            other => panic!("unexpected pipeline value for {key}: {other:?}"),
        };
        let direct = client.get_ttl(key).await.unwrap();
        assert_eq!(
            from_pipeline, direct,
            "pipeline TTL diverged from per-key TTL for {key}"
        );
    }

    // -1 is "no expiry", -2 is "missing key"; the caller maps both.
    assert_eq!(pipelined[0], RedisValue::Integer(-1));
    assert_eq!(pipelined[2], RedisValue::Integer(-2));

    for key in ["pipe:persistent", "pipe:expiring"] {
        client.del(vec![key]).await.unwrap();
    }
}

#[tokio::test]
#[ignore]
async fn test_read_only_client_blocks_writes_but_allows_reads() {
    let mut config = test_config();
    config.readonly = true;
    let mut client = StandaloneClient::new(config);
    client.connect().await.unwrap();

    client
        .execute("SET", vec!["ro:key".into(), "v".into()])
        .await
        .unwrap_err();
    client.del(vec!["ro:key"]).await.unwrap_err();
    assert!(client.rename("a", "b").await.is_err());
    assert!(client.persist("ro:key").await.is_err());
    assert!(client.set_ttl("ro:key", 60).await.is_err());

    // Reads still have to work, otherwise a read-only connection is useless.
    assert!(client.get_type("ro:missing").await.is_ok());
    assert!(client.get_ttl("ro:missing").await.is_ok());
    assert!(client
        .execute("GET", vec!["ro:missing".into()])
        .await
        .is_ok());
    assert!(client
        .execute_pipeline(vec![(
            "TTL".to_string(),
            vec!["ro:missing".to_string()]
        )])
        .await
        .is_ok());
}

#[tokio::test]
#[ignore]
async fn test_read_only_client_rejects_unsafe_pipeline() {
    let mut config = test_config();
    config.readonly = true;
    let mut client = StandaloneClient::new(config);
    client.connect().await.unwrap();

    // A read scan builds a TTL-only pipeline; a mixed batch must be rejected
    // outright rather than partially applied.
    client
        .execute_pipeline(vec![
            ("TTL".to_string(), vec!["k".to_string()]),
            ("DEL".to_string(), vec!["k".to_string()]),
        ])
        .await
        .unwrap_err();
}
