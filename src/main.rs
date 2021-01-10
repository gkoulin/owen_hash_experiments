#![allow(unused)]

use rayon::prelude::*;
use std::fs::File;

mod halton;
mod r2;
mod sobol;

const RESOLUTION: usize = 320;

type Stats = ([[f64; 32]; 32], [[f64; 32]; 32]);
const STATS_ZERO: Stats = ([[0.0; 32]; 32], [[0.0; 32]; 32]);

fn main() {
    // Set rayon per-thread stack size, because by default it's stupid small.
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 8)
        .build_global()
        .unwrap();

    // Parse command line arguments.
    let args = clap::App::new("Sample Testing")
        .version("0.123456789")
        .about("")
        .arg(clap::Arg::with_name("test").long("test"))
        .arg(clap::Arg::with_name("test_image").long("img"))
        .arg(clap::Arg::with_name("optimize").long("opt"))
        .arg(clap::Arg::with_name("reference").long("ref"))
        .arg(
            clap::Arg::with_name("number")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    // Pick what to do based on command line arguments.
    if args.is_present("test") {
        do_test(args.is_present("test_image"));
    } else if args.is_present("optimize") {
        let rounds = args.value_of("number").unwrap_or("2500").parse().unwrap();
        do_optimization(rounds);
    } else {
        const SETS: &[u32] = &[256, 1024, 4096];
        const PLOT_RADIUS: usize = 2;

        let image_count = args.value_of("number").unwrap_or("4").parse().unwrap();

        for seed in 0..image_count {
            let width = RESOLUTION * SETS.len();
            let height = RESOLUTION;
            let mut image = vec![0xffu8; width * height * 4];
            let mut file = File::create(&format!("{:02}.png", seed)).unwrap();

            let mut plot = |x: usize, y: usize| {
                let min_x = x.saturating_sub(PLOT_RADIUS);
                let min_y = y.saturating_sub(PLOT_RADIUS);
                let max_x = (x + PLOT_RADIUS + 1).min(width);
                let max_y = (y + PLOT_RADIUS + 1).min(height);

                for yy in min_y..max_y {
                    for xx in min_x..max_x {
                        let x2 = x as isize - xx as isize;
                        let y2 = y as isize - yy as isize;
                        if (((x2 * x2) + (y2 * y2)) as f64).sqrt() <= PLOT_RADIUS as f64 {
                            image[(yy * width + xx) * 4] = 0x00;
                            image[(yy * width + xx) * 4 + 1] = 0x00;
                            image[(yy * width + xx) * 4 + 2] = 0x00;
                            image[(yy * width + xx) * 4 + 3] = 0xFF;
                        }
                    }
                }
            };

            let scramble_1 = seed * 2;
            let scramble_2 = seed * 2 + 1;
            for si in 0..SETS.len() {
                for i in 0..SETS[si] {
                    let (x, y) = if args.is_present("reference") {
                        (
                            sobol::sample_owen_slow(0, i, scramble_1),
                            sobol::sample_owen_slow(1, i, scramble_2),
                        )
                    } else {
                        (
                            sobol::sample_owen(0, i, scramble_1),
                            sobol::sample_owen(1, i, scramble_2),
                        )
                    };

                    plot(
                        (x * (RESOLUTION - 1) as f32) as usize + (RESOLUTION * si),
                        (y * (RESOLUTION - 1) as f32) as usize,
                    );
                }
            }

            png_encode_mini::write_rgba_from_u8(&mut file, &image, width as u32, height as u32);
        }
    }
}

//=======================================================================
// SUB-COMMANDS
//=======================================================================

fn do_test(with_image: bool) {
    let rand_ints: Vec<u32> = (0..4096).map(|_| rand::random::<u32>()).collect();

    println!("{:08x?}", &rand_ints[..8]);

    for &hash_rounds in [1, 2, 3, 4, 8, 16, 32, 64, 128, 256].iter() {
        let avalanche_stats = measure_avalanche(
            |n, seed| {
                let mut n = n;

                // // LK version
                // n = n.wrapping_add(seed);
                // n ^= n.wrapping_mul(0x6c50b47c);
                // n ^= n.wrapping_mul(0xb82f1e52);
                // n ^= n.wrapping_mul(0xc7afe638);
                // n ^= n.wrapping_mul(0x8d22f6e6);

                // // LK rounds
                // n += hash_u32(seed, 0);
                // // n ^= hash_u32(seed, 0);
                // for i in 0..hash_rounds {
                //     n ^= n.wrapping_mul(rand_ints[i] & !1);
                // }

                // // Improved version 3
                // n = n.wrapping_add(hash_u32(seed, 0));
                // for p in rand_ints.chunks(2).take(hash_rounds) {
                //     n = n.wrapping_mul(p[0] | 1);
                //     n ^= p[1];
                // }

                // // Improved v4
                // n = n.wrapping_add(hash_u32(seed, 0));
                // // n ^= hash_u32(seed, 0);
                // for p in rand_ints.chunks(2).take(hash_rounds) {
                //     n ^= n.wrapping_mul(p[0] & !1);
                //     n = n.wrapping_mul(p[1] | 1);
                // }

                // // Improved v4 with optimized constants.
                // let perms: &[(u32, u32)] = &[
                //     (0xa2d0f65a, 0x22bbe06d),
                //     (0xeb8e0374, 0x0c8c8841),
                //     (0xed3a0b98, 0xd1f0ca7b),
                // ];
                // n = n.wrapping_add(hash_u32(seed, 0));
                // // n ^= hash_u32(seed, 0);
                // for i in 0..hash_rounds.min(perms.len()) {
                //     n ^= n.wrapping_mul(perms[i].0 & !1);
                //     n = n.wrapping_mul(perms[i].1 | 1);
                // }

                // // Improved v5
                // let scramble = hash_u32(seed, 0);
                // let scramble2 = hash_u32(seed, 1);
                // n = n.wrapping_add(scramble);
                // for i in 0..hash_rounds {
                //     n = n.wrapping_mul(scramble2 | 1);
                //     n ^= n.wrapping_mul(rand_ints[i*2] & !1);
                //     n = n.wrapping_mul(rand_ints[i*2+1] | 1);
                // }

                // // Improved v5 with optimized constants.
                // let perms: &[(u32, u32)] = &[
                //     (0xfad85de6, 0xf6db595b),
                //     (0x17ebb038, 0xe100f46f),
                //     (0x09e4ac1a, 0xe1d8c1ff),
                // ];
                // let scramble = hash_u32(seed, 0);
                // let scramble2 = hash_u32(seed, 1);
                // n = n.wrapping_add(scramble);
                // for i in 0..hash_rounds.min(perms.len()) {
                //     n = n.wrapping_mul(scramble2 | 1);
                //     n ^= n.wrapping_mul(perms[i].0 & !1);
                //     n = n.wrapping_mul(perms[i].1 | 1);
                // }

                // // Add Xor version
                // // n = n.wrapping_mul(hash_u32(seed, 0) | 1);
                // n += hash_u32(seed, 0);
                // for p in rand_ints.chunks(2).cycle().take(hash_rounds) {
                //     n = n.wrapping_add(p[0]);
                //     n ^= p[1];
                // }

                n = n.reverse_bits();
                n = sobol::owen_scramble_slow(n, seed);
                n = n.reverse_bits();

                n
            },
            1 << 23,
            true,
        );

        // Print stats.
        println!("Rounds: {}", hash_rounds);
        print_stats(avalanche_stats);
        println!();

        // Write avalanche image.
        if with_image {
            write_avalanche_image(
                avalanche_stats,
                &mut File::create(&format!("rounds_{:04}.png", hash_rounds)).unwrap(),
            );
        }
    }
}

fn do_optimization(rounds: usize) {
    let (perms, stats) = optimize(
        rounds,
        2, // Simultaneous candidates to use.
        0, // Bits to ignore.
        // Generate
        || {
            [
                0xfad85de6,
                0xf6db595b,
                0x17ebb038,
                0xe100f46f,
                0x09e4ac1a,
                0xe1d8c1ff,
                rand::random::<u32>() & !1,
                rand::random::<u32>() | 1,
            ]
        },
        // Mutate
        |n| {
            // Only mutate the last two items.
            let idx = n.len() - 2 + (rand::random::<u8>() as usize % 2);
            let mut n = n;
            n[idx] = n[idx] ^ (1 << (rand::random::<u8>() % 32).max(1));
            n
        },
        // Execute
        |mut a, n, s| {
            // a = a.wrapping_add(s);
            // for p in n.chunks(2) {
            //     a ^= a.wrapping_mul(p[0] & !1);
            //     a = a.wrapping_mul(p[1] | 1);
            // }
            // a

            let s2 = hash_u32(s, 0);
            a = a.wrapping_add(s);
            for p in n.chunks(2) {
                a = a.wrapping_mul(s2 | 1);
                a ^= a.wrapping_mul(p[0] & !1);
                a = a.wrapping_mul(p[1] | 1);
            }
            a
        },
    );

    for x in perms.iter() {
        println!("{:032b}", *x);
    }
    print!("[");
    for p in perms.iter() {
        print!("0x{:08x?}, ", *p);
    }
    println!("]");
    print_stats(stats);
}

//=======================================================================
// UTILS
//=======================================================================

fn hash_u32(n: u32, seed: u32) -> u32 {
    // Fast version.
    // From https://github.com/skeeto/hash-prospector
    let mut n = n.wrapping_add(seed.wrapping_mul(0x736caf6f));
    n ^= n >> 17;
    n = n.wrapping_mul(0xed5ad4bb);
    n ^= n >> 11;
    n = n.wrapping_mul(0xac4c1b51);
    n ^= n >> 15;
    n = n.wrapping_mul(0x31848bab);
    n ^= n >> 14;
    n

    // // Slow version, for comparison.
    // let mut in_bytes = [0u8; 8];
    // let mut out_bytes = [0u8; 4];
    // &in_bytes[..4].copy_from_slice(&seed.to_le_bytes());
    // &in_bytes[4..].copy_from_slice(&n.to_le_bytes());
    // &out_bytes.copy_from_slice(&blake3::hash(&in_bytes).as_bytes()[..4]);
    // u32::from_le_bytes(out_bytes)
}

fn print_stats(stats: Stats) {
    // Calculate reduced stats
    let mut reduced_stats = [0.0f64; 32]; // (avg, max)
    for bit_in in 0..32 {
        for bit_out in (bit_in + 1)..32 {
            reduced_stats[bit_out] += stats.0[bit_in][bit_out] / bit_out as f64;
        }
    }

    // Calculate average bias.
    let mut avg_bias = 0.0;
    for bit_in in 0..32 {
        for bit_out in (bit_in + 1)..32 {
            avg_bias += stats.0[bit_in][bit_out];
        }
    }
    avg_bias /= (32 * 31 / 2) as f64;

    // Print info.
    // println!("{:0.2?}", reduced_stats);
    println!("Average bias: {:0.3}", avg_bias);
}

fn write_avalanche_image(stats: Stats, file: &mut File) {
    const BIT_PIXEL_SIZE: usize = 8;
    const WIDTH: usize = BIT_PIXEL_SIZE * 32 * 2;
    const HEIGHT: usize = BIT_PIXEL_SIZE * 32;
    let mut image = vec![0x00u8; 4 * WIDTH * HEIGHT];
    let mut plot = |x: usize, y: usize, color: u8| {
        let min_x = x * BIT_PIXEL_SIZE;
        let min_y = y * BIT_PIXEL_SIZE;
        let max_x = min_x + BIT_PIXEL_SIZE;
        let max_y = min_y + BIT_PIXEL_SIZE;

        for y in min_y..max_y {
            for x in min_x..max_x {
                image[(y * WIDTH + x) * 4] = color;
                image[(y * WIDTH + x) * 4 + 1] = color;
                image[(y * WIDTH + x) * 4 + 2] = color;
                image[(y * WIDTH + x) * 4 + 3] = 0xFF;
            }
        }
    };

    for bit_in in 0..32 {
        for bit_out in 0..32 {
            let color_avalanche = (stats.0[bit_in][bit_out].min(1.0).max(0.0) * 255.0) as u8;
            let color_tree = (stats.1[bit_in][bit_out].min(1.0).max(0.0) * 255.0) as u8;
            plot(bit_out, bit_in, color_avalanche);
            plot(bit_out + 32, bit_in, color_tree);
        }
    }
    png_encode_mini::write_rgba_from_u8(file, &image, WIDTH as u32, HEIGHT as u32);
}

fn optimize<T: Copy, F1, F2, F3>(
    rounds: usize,
    candidates: usize,
    ignore_bits: usize, // Ignore the lowest N bits when scoring.
    generate: F1,
    mutate: F2,
    execute: F3,
) -> (T, Stats)
where
    T: Sync,
    F1: Fn() -> T + Sync,
    F2: Fn(T) -> T + Sync,
    F3: Fn(u32, T, u32) -> u32 + Sync, // (input, hash_constants, seed) -> hash
{
    let mut current: Vec<_> = (0..candidates)
        .map(|_| (generate(), std::f64::INFINITY, STATS_ZERO))
        .collect();

    println!();
    for round in 0..rounds {
        print!("\rround {}/{}", round, rounds);
        let do_score = |a| {
            const STAT_ROUNDS: u32 = 1 << 14;
            let stats = measure_avalanche(|n, s| execute(n, a, s), STAT_ROUNDS, false);

            // Calculate score.
            let mut score = 0.0;

            // Tree seeding bias metric
            for x in 0..32 {
                for y in (x + 1)..32 {
                    let diff = stats.1[x][y] - 0.5;
                    score += diff * diff;
                }
            }

            // // Avalanche bias metric, trying to simply minimize bias in all
            // // seeded trees as much as possible.
            // for bit_out in 0..32 {
            //     for bit_in in 0..bit_out {
            //         let bias = stats.0[bit_in][bit_out];
            //         score += bias * bias;
            //     }
            // }

            // Avalanche bias metric, trying to match the expected bias of a
            // proper full Owen scramble.
            const TARGET_BIAS: [f64; 32] = [
                0.0, 1.0, 0.5, 0.375, 0.273437, 0.19638, 0.139949, 0.099346, 0.070386, 0.049819,
                0.035244, 0.024927, 0.017628, 0.012466, 0.008815, 0.006233, 0.004407, 0.003117,
                0.002204, 0.001558, 0.001102, 0.000779, 0.000551, 0.000390, 0.000275, 0.000195,
                0.000138, 0.000097, 0.000069, 0.000049, 0.000034, 0.000024,
            ];
            for bit_out in 0..32 {
                for bit_in in 0..bit_out {
                    let diff = stats.0[bit_in][bit_out] - TARGET_BIAS[bit_out];
                    score += diff * diff;
                }
            }

            (score, stats)
        };

        current.sort_unstable_by(|x, y| x.1.partial_cmp(&y.1).unwrap());

        for i in 0..candidates {
            let n = if i < (candidates / 2) {
                mutate(current[i].0)
            } else {
                generate()
            };
            let (score, stats) = do_score(n);
            if score < current[i].1 {
                current[i] = (n, score, stats);
            }
        }
    }
    println!();

    (current[0].0, current[0].2)
}

/// Measures the avalanche bias of the provided hash function.
///
/// The returned 2d array contains (average bias, max bias) tuples for each
/// bit pairing.  It's accessed as [input_bit][output_bit].
fn measure_avalanche<F>(hash: F, rounds: u32, print_progress: bool) -> Stats
where
    F: Fn(u32, u32) -> u32 + Sync, // (input, seed) -> output
{
    use std::io::Write;

    // Break up the rounds into chunks that we can hoist off to different
    // threads.
    let sub_rounds = 256;
    let loop_rounds = (rounds / sub_rounds) + ((rounds % sub_rounds) != 0) as u32;
    let rounds = loop_rounds * sub_rounds;

    if print_progress {
        print!("Progress..");
        std::io::stdout().flush();
    }
    let data = (0..loop_rounds)
        .into_par_iter()
        .map(|lr| {
            if print_progress && (lr % (loop_rounds / 53)) == 0 {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                out.write_all(b".");
                out.flush();
            }

            // Run tests and collect data.
            let seed = rand::random::<u32>();
            let mut data = STATS_ZERO;
            for i in 0..sub_rounds {
                let input_1 = rand::random::<u32>();
                let output_1 = hash(input_1, seed);

                // Avalanche bias.
                for bit_in in 0..32 {
                    let input_2 = input_1 ^ (1 << bit_in);
                    let output_2 = hash(input_2, seed);
                    let diff_1 = output_1 ^ output_2;
                    for bit_out in 0..32 {
                        if (diff_1 & (1 << bit_out)) != 0 {
                            data.0[bit_in][bit_out] += 1.0;
                        }
                    }
                }

                // Tree seeding bias.
                let seed2 = rand::random::<u32>();
                let input_3 = rand::random::<u32>();
                let output_3 = hash(input_3, seed2);
                let input_4 = rand::random::<u32>();
                let output_4 = hash(input_4, seed2);
                let mut x = (input_3 ^ input_4 ^ output_3 ^ output_4).reverse_bits() as usize;
                let mut y = (input_3 ^ input_4).reverse_bits() as usize;
                x = x >> 26;
                y = (y >> 26) - 32;
                if y < 32 {
                    // We lose some data by exluding samples, but in practice
                    // it seems to be redundant anyway.  But take this out of
                    // the of the if statement if something seems inconsistent
                    // in the output.
                    data.1[x & 0b11111][y & 0b11111] += 1.0;
                }
            }

            // Process data.
            for i in 0..32 {
                for j in 0..32 {
                    data.0[i][j] = (data.0[i][j] - (0.5 * sub_rounds as f64)).abs();
                }
            }

            data
        })
        .reduce(
            || STATS_ZERO,
            |mut a, b| {
                for i in 0..32 {
                    for j in 0..32 {
                        a.0[i][j] += b.0[i][j];
                        a.1[i][j] += b.1[i][j];
                    }
                }
                a
            },
        );
    if print_progress {
        print!(
            "\r                                                                                \r"
        );
    }

    let mut stats = STATS_ZERO;
    for i in 0..32 {
        for j in 0..32 {
            stats.0[i][j] += data.0[i][j] * 2.0 / rounds as f64;
            stats.1[i][j] += data.1[i][j] / rounds as f64 * 32.0 * 32.0;
        }
    }

    stats
}
