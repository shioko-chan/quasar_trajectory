use criterion::{criterion_group, criterion_main, Criterion};
use messages::new_tube;

fn bench_tube(c: &mut Criterion) {
    c.bench_function("tube", |b| {
        b.iter({
            || {
                let (mut sender, receiver) = new_tube::<Vec<u8>>();
                let _ = sender.get_send_buffer();
                sender.send().unwrap();
                let received = receiver.recv().unwrap();
                receiver.recycle(received).unwrap();
                let _ = sender.get_send_buffer();
                sender.send().unwrap();
                let received = receiver.recv().unwrap();
                receiver.recycle(received).unwrap();
            }
        });
    });
}

criterion_group!(benches, bench_tube);
criterion_main!(benches);
