// window of one is just consecutive commits

use hyper_ast_cvs_git::preprocessed::PreProcessedRepository;
use std::{
    env,
    io::{BufWriter, Write},
    path::PathBuf,
    str::FromStr, fs::File,
};

use hyper_ast_benchmark_diffs::window_combination::windowed_commits_compare;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let args: Vec<String> = env::args().collect();
    log::warn!("args: {:?}", args);
    let repo_name = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon"); //"openjdk/jdk";//"INRIA/spoon";
    let before = args.get(2).map_or("", |x| x);
    let after = args.get(3).map_or("", |x| x);
    let out = args.get(4).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            Some(PathBuf::from_str(x).unwrap())
        }
    });
    let preprocessed = PreProcessedRepository::new(&repo_name);
    // let (before, after) = (
    //         // "e01840e86db739374c9c4eb84a948b24ca017d8e", // parent
    //         "cf626243f627cca3d52ff073ebc53eca9595d4b5", // git log --pretty=format:"%H" | head
    //         // # classic gumtree
    //         // only mappings // java -cp  gumtree.spoon.AstComparator /tmp/spoon  187.53s user 3.72s system 174% cpu 1:49.78 total
    //         // diff // java -cp  gumtree.spoon.AstComparator /tmp/spoon  198.09s user 2.94s system 183% cpu 1:49.42 total
    //         // # lazy
    //         // cargo run --bin=window_combination --release  59.67s user 0.34s system 99% cpu 1:00.45 total
    //         // cargo run --bin=window_combination --release  61.64s user 0.35s system 99% cpu 1:02.55 total

    //         "00dc4b0b13622dfeccb8d67757422c5bd1bf1e38",
    //     );
    // whole();
    inc(preprocessed, before, after, out);
}

/// incrementally build each commits and compute diffs
/// ie. interlace building a commit and computing diff with its child commit
/// nb. 2 commits need to be build before doing the first diff
fn inc(
    mut preprocessed: PreProcessedRepository,
    before: &str,
    after: &str,
    out: Option<PathBuf>,
) {
    let batch_id = format!("{}:({},{})", &preprocessed.name, before, after);
    dbg!(batch_id);
    // let mu = memusage_linux();
    // let hyperast_size = memusage_linux() - mu;
    // log::warn!("hyperAST size: {}", hyperast_size);
    // log::warn!("batch_id: {batch_id}");
    // let mu = memusage_linux();
    // log::warn!("total memory used {mu}");
    // preprocessed.purge_caches();
    // let mu = mu - memusage_linux();
    // log::warn!("cache size: {mu}");
    // log::warn!(
    //     "commits ({}): {:?}",
    //     preprocessed.commits.len(),
    //     processing_ordered_commits
    // );
    // let c_len = processing_ordered_commits.len();

    let mut buf = out
    .map(|out| File::create(out).unwrap())
    .map(|file| BufWriter::with_capacity(4 * 8 * 1024, file));
    if let Some(buf) = &mut buf {
        writeln!(
            buf,
            "input,src_s,dst_s,src_heap,dst_heap,src_t,dst_t,mappings,diff_t,changes"
        )
        .unwrap();
        buf.flush().unwrap();
    }
    use hyper_ast_gen_ts_java::utils::memusage_linux;
    let mut curr = after.to_string();
    for i in 0..100 {
        if curr == before {
            break;
        }
        let processing_ordered_commits = preprocessed.pre_process_with_limit(
            &mut hyper_ast_cvs_git::git::fetch_github_repository(&preprocessed.name),
            "",
            &curr,
            "",
            2,
        );
        let oid_src = processing_ordered_commits[1];
        let oid_dst = processing_ordered_commits[0];
        assert_eq!(curr,oid_dst.to_string());
        log::warn!("diff of {oid_src} and {oid_dst}");

        let stores = &preprocessed.processor.main_stores;

        let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
        let time_src = commit_src.1.processing_time();
        let src_tr = commit_src.1.ast_root;
        use hyper_ast::types::WithStats;
        let src_s = stores.node_store.resolve(src_tr).size();

        let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
        let time_dst = commit_dst.1.processing_time();
        let dst_tr = commit_dst.1.ast_root;
        let dst_s = stores.node_store.resolve(dst_tr).size();

        let hyperast = hyper_ast_cvs_git::no_space::as_nospaces(stores);

        let mu = memusage_linux();
        let lazy = hyper_diff::algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr);
        let summarized_lazy = &lazy.summarize();
        use hyper_diff::algorithms::ComputeTime;
        let total_lazy_t: f64 = summarized_lazy.time();
        dbg!(summarized_lazy);
        log::warn!("ed+mappings size: {}", memusage_linux() - mu);
        log::warn!("done computing diff {i}");
        if let Some(buf) = &mut buf {
            writeln!(
                buf,
                "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{}",
                src_s,
                dst_s,
                Into::<isize>::into(&commit_src.1.memory_used()),
                Into::<isize>::into(&commit_dst.1.memory_used()),
                time_src,
                time_dst,
                summarized_lazy.mappings,
                total_lazy_t,
                summarized_lazy.actions.map_or(-1,|x|x as isize),
            )
            .unwrap();
            buf.flush().unwrap();
        }

        curr = oid_src.to_string();
    }
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());
}

fn whole(
    mut preprocessed: PreProcessedRepository,
    before: &str,
    after: &str,
    buf: Option<BufWriter<File>>,
) {
    let window_size = 2;
    let batch_id = format!("{}:({},{})", &preprocessed.name, before, after);
    let mu = memusage_linux();
    let processing_ordered_commits = preprocessed.pre_process_with_limit(
        &mut hyper_ast_cvs_git::git::fetch_github_repository(&preprocessed.name),
        before,
        after,
        "",
        10,
    );
    let hyperast_size = memusage_linux() - mu;
    log::warn!("hyperAST size: {}", hyperast_size);
    log::warn!("batch_id: {batch_id}");
    let mu = memusage_linux();
    log::warn!("total memory used {mu}");
    preprocessed.purge_caches();
    let mu = mu - memusage_linux();
    log::warn!("cache size: {mu}");
    log::warn!(
        "commits ({}): {:?}",
        preprocessed.commits.len(),
        processing_ordered_commits
    );
    let mut i = 0;
    let c_len = processing_ordered_commits.len();
    use hyper_ast_gen_ts_java::utils::memusage_linux;
    for c in (0..c_len - 1).map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)]) {
        let oid_src = c[0];
        for oid_dst in &c[1..] {
            log::warn!("diff of {oid_src} and {oid_dst}");

            let stores = &preprocessed.processor.main_stores;

            let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
            let src_tr = commit_src.1.ast_root;

            let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
            let dst_tr = commit_dst.1.ast_root;

            let hyperast = hyper_ast_cvs_git::no_space::as_nospaces(stores);

            let mu = memusage_linux();
            let lazy = hyper_diff::algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr);
            let summarized_lazy = &lazy.summarize();
            dbg!(summarized_lazy);
            log::warn!("ed+mappings size: {}", memusage_linux() - mu);
        }
        log::warn!("done computing diff {i}");
        i += 1;
    }
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());
}
