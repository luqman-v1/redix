// Pongolong: Tests use #[ignore] — run with `cargo test -- --ignored` when Redis is up
use redix_app_lib::redis::{StandaloneClient, RedisClient, RedisValue};
use redix_app_lib::config::ConnectionConfig;

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
    let result = client.execute("SET", vec!["test_key".into(), "hello".into()]).await.unwrap();
    assert_eq!(result, RedisValue::Status("OK".to_string()));

    // GET
    let result = client.execute("GET", vec!["test_key".into()]).await.unwrap();
    assert_eq!(result, RedisValue::String("hello".to_string()));

    // DEL
    let result = client.execute("DEL", vec!["test_key".into()]).await.unwrap();
    assert_eq!(result, RedisValue::Integer(1));

    // GET after DEL → nil
    let result = client.execute("GET", vec!["test_key".into()]).await.unwrap();
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
        client.execute("DEL", vec![format!("scan_test:{}", i)]).await.unwrap();
    }
}
