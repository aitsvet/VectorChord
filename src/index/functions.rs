// This software is licensed under a dual license model:
//
// GNU Affero General Public License v3 (AGPLv3): You may use, modify, and
// distribute this software under the terms of the AGPLv3.
//
// Elastic License v2 (ELv2): You may also use, modify, and distribute this
// software under the Elastic License v2, which has specific restrictions.
//
// We welcome any commercial collaboration or support. For inquiries
// regarding the licenses, please contact us at:
// vectorchord-inquiry@tensorchord.ai
//
// Copyright (c) 2025 TensorChord Inc.

use crate::index::storage::PostgresRelation;
use crate::index::vchordrq::dispatch::CentroidResult;
use crate::recorder::dump;
use pgrx::iter::{SetOfIterator, TableIterator};
use pgrx::pg_sys::Oid;
use pgrx_catalog::{PgAm, PgClass, PgClassRelkind};

#[pgrx::pg_extern(sql = "")]
fn _vchordg_prewarm(indexrelid: Oid) -> String {
    let pg_am = PgAm::search_amname(c"vchordg").unwrap();
    let Some(pg_am) = pg_am.get() else {
        pgrx::error!("vchord is not installed");
    };
    let pg_class = PgClass::search_reloid(indexrelid).unwrap();
    let Some(pg_class) = pg_class.get() else {
        pgrx::error!("the relation does not exist");
    };
    if pg_class.relkind() != PgClassRelkind::Index {
        pgrx::error!("the relation {:?} is not an index", pg_class.relname());
    }
    if pg_class.relam() != pg_am.oid() {
        pgrx::error!("the index {:?} is not a vchordg index", pg_class.relname());
    }
    let relation = Index::open(indexrelid, pgrx::pg_sys::AccessShareLock as _);
    let opfamily = unsafe { crate::index::vchordg::opclass::opfamily(relation.raw()) };
    let index = unsafe { PostgresRelation::new(relation.raw()) };
    crate::index::vchordg::dispatch::prewarm(opfamily, &index)
}

#[pgrx::pg_extern(sql = "")]
fn _vchordrq_prewarm(indexrelid: Oid, height: i32) -> String {
    let pg_am = PgAm::search_amname(c"vchordrq").unwrap();
    let Some(pg_am) = pg_am.get() else {
        pgrx::error!("vchord is not installed");
    };
    let pg_class = PgClass::search_reloid(indexrelid).unwrap();
    let Some(pg_class) = pg_class.get() else {
        pgrx::error!("the relation does not exist");
    };
    if pg_class.relkind() != PgClassRelkind::Index {
        pgrx::error!("the relation {:?} is not an index", pg_class.relname());
    }
    if pg_class.relam() != pg_am.oid() {
        pgrx::error!("the index {:?} is not a vchordrq index", pg_class.relname());
    }
    let relation = Index::open(indexrelid, pgrx::pg_sys::AccessShareLock as _);
    let opfamily = unsafe { crate::index::vchordrq::opclass::opfamily(relation.raw()) };
    let index = unsafe { PostgresRelation::new(relation.raw()) };
    crate::index::vchordrq::dispatch::prewarm(opfamily, &index, height)
}

struct Index {
    raw: *mut pgrx::pg_sys::RelationData,
    lockmode: pgrx::pg_sys::LOCKMODE,
}

impl Index {
    fn open(indexrelid: Oid, lockmode: pgrx::pg_sys::LOCKMASK) -> Self {
        Self {
            raw: unsafe { pgrx::pg_sys::index_open(indexrelid, lockmode) },
            lockmode,
        }
    }
    fn raw(&self) -> *mut pgrx::pg_sys::RelationData {
        self.raw
    }
}

impl Drop for Index {
    fn drop(&mut self) {
        unsafe {
            pgrx::pg_sys::index_close(self.raw, self.lockmode);
        }
    }
}

#[pgrx::pg_extern(sql = "")]
fn _vchordrq_sampled_values(indexrelid: Oid) -> SetOfIterator<'static, String> {
    let pg_am = PgAm::search_amname(c"vchordrq").unwrap();
    let Some(pg_am) = pg_am.get() else {
        pgrx::error!("vchord is not installed");
    };
    let pg_class = PgClass::search_reloid(indexrelid).unwrap();
    let Some(pg_class) = pg_class.get() else {
        pgrx::error!("the relation does not exist");
    };
    if pg_class.relkind() != PgClassRelkind::Index {
        pgrx::error!("the relation {:?} is not an index", pg_class.relname());
    }
    if pg_class.relam() != pg_am.oid() {
        pgrx::error!("the index {:?} is not a vchordrq index", pg_class.relname());
    }
    let _relation = Index::open(indexrelid, pgrx::pg_sys::AccessShareLock as _);
    let queries = dump(indexrelid.to_u32());
    SetOfIterator::new(queries)
}

#[pgrx::pg_extern(sql = "")]
fn _vchordrq_list_centroids(
    indexrelid: Oid,
) -> TableIterator<
    'static,
    (
        pgrx::name!(level, i32),
        pgrx::name!(id, i32),
        pgrx::name!(centroid, String),
    ),
> {
    let pg_am = PgAm::search_amname(c"vchordrq").unwrap();
    let Some(pg_am) = pg_am.get() else {
        pgrx::error!("vchord is not installed");
    };
    let pg_class = PgClass::search_reloid(indexrelid).unwrap();
    let Some(pg_class) = pg_class.get() else {
        pgrx::error!("the relation does not exist");
    };
    if pg_class.relkind() != PgClassRelkind::Index {
        pgrx::error!("the relation {:?} is not an index", pg_class.relname());
    }
    if pg_class.relam() != pg_am.oid() {
        pgrx::error!("the index {:?} is not a vchordrq index", pg_class.relname());
    }
    let relation = Index::open(indexrelid, pgrx::pg_sys::AccessShareLock as _);
    let opfamily = unsafe { crate::index::vchordrq::opclass::opfamily(relation.raw()) };
    let index = unsafe { PostgresRelation::new(relation.raw()) };
    let centroids = crate::index::vchordrq::dispatch::list_centroids(opfamily, &index);
    TableIterator::new(
        centroids
            .into_iter()
            .map(|c: CentroidResult| (c.level, c.id, c.vector)),
    )
}
