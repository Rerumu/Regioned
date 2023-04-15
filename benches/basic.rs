use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};

use regioned::data_flow::{
	graph::Graph,
	node::{Compound, Node},
};

const NUM_ELEMENTS: u64 = 2048;

struct NoOp(u64);

pub fn bench_add(c: &mut Criterion) {
	let mut group = c.benchmark_group("Add");

	group.throughput(Throughput::Elements(NUM_ELEMENTS));

	group.bench_function("Node", |b| {
		b.iter_with_large_drop(|| {
			let mut graph = Graph::new();

			for i in 0..NUM_ELEMENTS {
				let value = Node::Simple(NoOp(i));
				let id = graph.add_node(value);

				black_box(id);
			}

			graph
		});
	});

	group.bench_function("Phi", |b| {
		b.iter_with_large_drop(|| {
			let mut graph = Graph::<NoOp>::new();

			for _ in 0..NUM_ELEMENTS {
				let result = graph.add_compound(Compound::Phi);

				black_box(result);
			}

			graph
		});
	});
}

pub fn bench_remove(c: &mut Criterion) {
	let mut group = c.benchmark_group("Remove");

	group.throughput(Throughput::Elements(NUM_ELEMENTS));

	group.bench_function("Node", |b| {
		b.iter_batched_ref(
			|| {
				let mut graph = Graph::new();
				let mut indices = Vec::new();

				for i in 0..NUM_ELEMENTS {
					let value = Node::Simple(NoOp(i));

					indices.push(graph.add_node(value));
				}

				(graph, indices)
			},
			|(graph, indices)| {
				for &mut index in indices {
					graph.remove_node(index);
				}
			},
			BatchSize::LargeInput,
		);
	});

	group.bench_function("Phi", |b| {
		b.iter_batched_ref(
			|| {
				let mut graph = Graph::<NoOp>::new();
				let mut indices = Vec::new();

				for _ in 0..NUM_ELEMENTS {
					let result = graph.add_compound(Compound::Phi);

					indices.push(result.0);
				}

				(graph, indices)
			},
			|(graph, indices)| {
				for &mut index in indices {
					graph.remove_compound(index);
				}
			},
			BatchSize::LargeInput,
		);
	});
}

fn bench_iteration(c: &mut Criterion) {
	fn setup() -> Graph<NoOp> {
		let mut graph = Graph::new();

		for i in 0..NUM_ELEMENTS {
			let value = Node::Simple(NoOp(i));
			let _id = graph.add_node(value);
		}

		graph
	}

	let mut group = c.benchmark_group("Bulk");

	group.throughput(Throughput::Elements(NUM_ELEMENTS));

	group.bench_function("Node", |b| {
		b.iter_batched_ref(
			setup,
			|graph| {
				for node in &graph.nodes {
					black_box(node);
				}
			},
			BatchSize::LargeInput,
		);
	});

	group.bench_function("Clear", |b| {
		b.iter_batched_ref(setup, Graph::clear, BatchSize::LargeInput);
	});
}

criterion_group!(benches, bench_add, bench_remove, bench_iteration);
criterion_main!(benches);
