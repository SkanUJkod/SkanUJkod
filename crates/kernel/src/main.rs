use abi_stable::DynTrait;
use abi_stable::std_types::{RHashMap, RString, Tuple2};

use abi_stable::library::lib_header_from_path;
use plugin_interface::{BoxedPFResult, PFDependencies, PluginRef, QualPFID, UserParameters};
use std::collections::HashMap;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    dbg!(&args);
    assert_eq!(args.len(), 2);

    let source_path: RString = args[1].clone().into();
    let boxed_source_path = DynTrait::from_value(source_path);

    let user_params = {
        let mut user_params: UserParameters = RHashMap::new();
        user_params.insert("file_path".into(), boxed_source_path);
        user_params
    };

    let plugins_dir =
        std::env::var("PLUGINS_DIR").expect("PLUGINS_DIR env variable has to be defined");
    let connectors = std::fs::read_dir(plugins_dir)
        .expect("Failed to open plugin directory")
        .map(|a| a.expect("Failed to read file in plugin directory"))
        .map(|dir_entry| dir_entry.path())
        .map(|path| {
            assert!(path.is_file());
            dbg!(&path);
            let header = lib_header_from_path(&path).unwrap();
            let lib = header.init_root_module::<PluginRef>().unwrap();
            lib.funcs()()
        })
        .collect::<Vec<_>>();

    let conns_flat = connectors.iter().flatten().collect::<Vec<_>>();
    let pf_to_idx: HashMap<QualPFID, usize> = conns_flat
        .iter()
        .enumerate()
        .map(|(i, c)| (c.pf_id.clone(), i))
        .collect();

    let n = conns_flat.len();
    let adj_list = {
        let mut adj_list: Vec<Vec<usize>> = vec![vec![]; n];
        let get_idx = |pfid| pf_to_idx.get(pfid).unwrap();

        conns_flat
            .iter()
            .map(|c| (&c.pf_id, &c.pf_type.pf_dependencies))
            .map(|(id, dep_ids)| (get_idx(id), dep_ids.into_iter().map(get_idx)))
            .for_each(|(&idx, dep_idxs)| dep_idxs.for_each(|&dep_idx| adj_list[dep_idx].push(idx)));

        adj_list
    };
    let mut vis = vec![false; n];
    let topo_order = {
        let mut topo_order: Vec<usize> = Vec::with_capacity(n);
        for i in 0..n {
            if !vis[i] {
                dfs(i, &adj_list, &mut vis, &mut topo_order);
            }
        }
        topo_order.reverse();
        topo_order
    };
    dbg!(&topo_order);

    let mut results: RHashMap<QualPFID, BoxedPFResult> = RHashMap::new();
    for v in topo_order {
        let pfc = conns_flat[v];
        dbg!(&pfc);
        let pf = &pfc.pf;
        let results_lens = results
            .iter_mut()
            .filter(|Tuple2(k, _)| pfc.pf_type.pf_dependencies.contains(k))
            .map(|Tuple2(k, v)| Tuple2(k.clone(), v))
            .collect::<PFDependencies>();
        let result = pf.0(results_lens, &user_params);
        dbg!(&result);
        results.insert(pfc.pf_id.clone(), result);
    }

    println!("success");
}

fn dfs(v: usize, adj_list: &Vec<Vec<usize>>, vis: &mut Vec<bool>, toposort: &mut Vec<usize>) {
    vis[v] = true;
    for &u in &adj_list[v] {
        if !vis[u] {
            dfs(u, adj_list, vis, toposort);
        }
    }
    toposort.push(v);
}
