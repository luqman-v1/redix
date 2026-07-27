use redis::{cluster_async::ClusterConnection, aio::MultiplexedConnection};

async fn test_pipe_cluster(mut conn: ClusterConnection) {
    let mut pipe = redis::pipe();
    pipe.cmd("GET").arg("key1");
    pipe.cmd("GET").arg("key2");
    let _: Vec<redis::Value> = pipe.query_async(&mut conn).await.unwrap();
}

async fn test_pipe_standalone(mut conn: MultiplexedConnection) {
    let mut pipe = redis::pipe();
    pipe.cmd("GET").arg("key1");
    pipe.cmd("GET").arg("key2");
    let _: Vec<redis::Value> = pipe.query_async(&mut conn).await.unwrap();
}
