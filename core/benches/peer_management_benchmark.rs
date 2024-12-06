use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, async_executor::FuturesExecutor};
use Nautilus_Core::record::{PeerManagement, PeerRecord};
use std::net::SocketAddr;

pub fn peer_management_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Peer Management");

    // Benchmark with asynchronous code
    group.bench_with_input(BenchmarkId::new("Add and Remove", "peers"), &(), |b, _| {
        b.to_async(FuturesExecutor).iter(|| async {
            let peer_manager = PeerManagement::new("test_peers_benchmark.json".to_string());

            let peer = PeerRecord {
                addr: "127.0.0.1:8000".parse::<SocketAddr>().unwrap(),
                peer_id: Some("peer_benchmark".to_string()),
                public_key: None,
                is_active: true,
                last_seen: None,
            };

            // Add and immediately remove a peer to test performance
            peer_manager.add_or_update_peer(peer.clone()).await;
            peer_manager.remove_peer("peer_benchmark").await;
        });
    });

    group.finish();
}

// Register the benchmark group
criterion_group!(benches, peer_management_benchmark);
criterion_main!(benches);
