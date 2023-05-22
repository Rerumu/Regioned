use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};

use regioned::data_flow::nodes::Nodes;

const NUM_ELEMENTS: u64 = 2048;

struct NoOp(u64);

pub fn bench_add(c: &mut Criterion) {
	let mut group = c.benchmark_group("Add");

	group.throughput(Throughput::Elements(NUM_ELEMENTS));

	group.bench_function("Node", |b| {
		b.iter_with_large_drop(|| {
			let mut nodes = Nodes::new();

			for i in 0..NUM_ELEMENTS {
				let id = nodes.add_simple(NoOp(i));

				black_box(id);
			}

			nodes
		});
	});

	group.bench_function("Phi", |b| {
		b.iter_with_large_drop(|| {
			let mut nodes = Nodes::<NoOp>::new();

			for _ in 0..NUM_ELEMENTS {
				let result = nodes.add_phi();

				black_box(result);
			}

			nodes
		});
	});
}

pub fn bench_remove(c: &mut Criterion) {
	let mut group = c.benchmark_group("Remove");

	group.throughput(Throughput::Elements(NUM_ELEMENTS));

	group.bench_function("Node", |b| {
		b.iter_batched_ref(
			|| {
				let mut nodes = Nodes::new();
				let mut indices = Vec::new();

				for i in 0..NUM_ELEMENTS {
					let id = nodes.add_simple(NoOp(i));

					indices.push(id);
				}

				(nodes, indices)
			},
			|(nodes, indices)| {
				for &mut index in indices {
					nodes.remove(index);
				}
			},
			BatchSize::LargeInput,
		);
	});

	group.bench_function("Phi", |b| {
		b.iter_batched_ref(
			|| {
				let mut nodes = Nodes::<NoOp>::new();
				let mut indices = Vec::new();

				for _ in 0..NUM_ELEMENTS {
					let result = nodes.add_phi();

					indices.push(result.0);
				}

				(nodes, indices)
			},
			|(nodes, indices)| {
				for &mut index in indices {
					nodes.remove(index);
				}
			},
			BatchSize::LargeInput,
		);
	});
}

fn bench_iteration(c: &mut Criterion) {
	fn setup() -> Nodes<NoOp> {
		let mut nodes = Nodes::new();

		for i in 0..NUM_ELEMENTS {
			let _id = nodes.add_simple(NoOp(i));
		}

		nodes
	}

	let mut group = c.benchmark_group("Bulk");

	group.throughput(Throughput::Elements(NUM_ELEMENTS));

	group.bench_function("Node", |b| {
		b.iter_batched_ref(
			setup,
			|nodes| {
				for node in nodes.iter() {
					black_box(node);
				}
			},
			BatchSize::LargeInput,
		);
	});

	group.bench_function("Clear", |b| {
		b.iter_batched_ref(setup, |nodes| nodes.clear(), BatchSize::LargeInput);
	});
}

criterion_group!(benches, bench_add, bench_remove, bench_iteration);
criterion_main!(benches);
