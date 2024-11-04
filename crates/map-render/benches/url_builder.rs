use criterion::{Criterion, black_box, criterion_group, criterion_main};
use map_render::bench_support::{PartialUrl, Tile};
use map_render::{MapStyle, Zoom};

fn setup(zoom: Zoom, style: MapStyle, tile_count: usize) -> (PartialUrl, Vec<Tile>) {
    let mut rng = rand::thread_rng();

    let partial = PartialUrl::bench_new(zoom, style);

    let max_tile = zoom.tiles_across();

    let mut tiles = Vec::with_capacity(tile_count);

    tiles.resize_with(tile_count, || Tile::random(&mut rng, max_tile));

    (partial, tiles)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const ZOOM: Zoom = Zoom::Fifteen;
    const STYLE: MapStyle = MapStyle::Base;

    const COUNT: usize = 5000;

    let params = format!("{STYLE:?}-z{:?}-{COUNT}", ZOOM as u8);

    let (partial, tiles) = black_box(setup(ZOOM, STYLE, COUNT));

    let res_id = criterion::BenchmarkId::new("complete_url", params);

    c.bench_with_input(res_id, &(partial, &tiles), |b, (partial, tiles)| {
        b.iter(|| {
            for tile in black_box(tiles.iter()) {
                let _ = black_box(partial.complete_url(black_box(*tile)));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
