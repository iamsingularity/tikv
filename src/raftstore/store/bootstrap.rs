// Copyright 2016 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use rocksdb::{DB, Writable};
use kvproto::raft_serverpb::StoreIdent;
use kvproto::metapb;
use raftstore::Result;
use super::keys;
use super::engine::{Iterable, Mutable};

const INIT_EPOCH_VER: u64 = 1;
const INIT_EPOCH_CONF_VER: u64 = 1;

// Bootstrap the store, the DB for this store must be empty and has no data.
pub fn bootstrap_store(engine: &DB, cluster_id: u64, store_id: u64) -> Result<()> {
    let mut ident = StoreIdent::new();

    let mut count: u32 = 0;
    try!(engine.scan(keys::MIN_KEY,
                     keys::MAX_KEY,
                     &mut |_, _| {
                         count += 1;
                         Ok(false)
                     }));

    if count > 0 {
        return Err(box_err!("store is not empty and has already had data."));
    }

    let ident_key = keys::store_ident_key();

    ident.set_cluster_id(cluster_id);
    ident.set_store_id(store_id);

    engine.put_msg(&ident_key, &ident)
}

// Write first region meta.
pub fn write_region(engine: &DB, region: &metapb::Region) -> Result<()> {
    try!(engine.put_msg(&keys::region_info_key(region.get_id()), region));
    Ok(())
}

// Clear first region meta.
pub fn clear_region(engine: &DB, region_id: u64) -> Result<()> {
    try!(engine.delete(&keys::region_info_key(region_id)));
    Ok(())
}

// Bootstrap first region.
pub fn bootstrap_region(engine: &DB, store_id: u64, region_id: u64) -> Result<metapb::Region> {
    let mut region = metapb::Region::new();
    region.set_id(region_id);
    region.set_start_key(keys::EMPTY_KEY.to_vec());
    region.set_end_key(keys::EMPTY_KEY.to_vec());
    region.mut_region_epoch().set_version(INIT_EPOCH_VER);
    region.mut_region_epoch().set_conf_ver(INIT_EPOCH_CONF_VER);

    region.mut_store_ids().push(store_id);

    try!(write_region(engine, &region));

    Ok(region)
}
