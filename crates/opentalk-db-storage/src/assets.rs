// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, Identifiable, Insertable, JoinOnDsl,
    NullableExpressionMethods as _, QueryDsl, Queryable,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use opentalk_database::{DbConnection, Paginate, Result};
use opentalk_types_common::{
    assets::{AssetId, AssetSorting},
    events::EventId,
    modules::ModuleId,
    order::Ordering,
    rooms::RoomId,
    tenants::TenantId,
    users::UserId,
};

use crate::schema::{assets, events, room_assets, rooms};

/// Diesel resource struct
#[derive(Debug, Clone, Queryable, Identifiable)]
pub struct Asset {
    pub id: AssetId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub namespace: Option<ModuleId>,
    pub kind: String,
    pub filename: String,
    pub tenant_id: TenantId,
    pub size: i64,
}

impl Asset {
    #[tracing::instrument(err, skip_all)]
    pub async fn get(conn: &mut DbConnection, id: AssetId, room_id: RoomId) -> Result<Self> {
        //FIXME: The inner_join below (as well as the room_id parameter) can be removed when assets have their own
        // permission check and don't rely on room permissions

        let query = assets::table
            .inner_join(
                room_assets::table.on(room_assets::asset_id
                    .eq(assets::id)
                    .and(room_assets::room_id.eq(room_id))),
            )
            .filter(assets::id.eq(id))
            .select(assets::all_columns);

        let resource: Asset = query.get_result(conn).await?;

        Ok(resource)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_ids_for_room(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Vec<AssetId>> {
        let query = room_assets::table
            .select(room_assets::asset_id)
            .filter(room_assets::room_id.eq(room_id));

        let assets = query.load(conn).await?;

        Ok(assets)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn count_all(conn: &mut DbConnection) -> Result<i64> {
        Ok(assets::table.count().get_result(conn).await?)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_for_room_paginated(
        conn: &mut DbConnection,
        room_id: RoomId,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<Self>, i64)> {
        let query = assets::table
            .inner_join(room_assets::table.on(room_assets::asset_id.eq(assets::id)))
            .filter(room_assets::room_id.eq(room_id))
            .select(assets::all_columns)
            .paginate_by(limit, page);

        let resources_with_total = query.load_and_count(conn).await?;

        Ok(resources_with_total)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_for_rooms_paginated(
        conn: &mut DbConnection,
        room_ids: &[RoomId],
        limit: i64,
        page: i64,
    ) -> Result<(Vec<Self>, i64)> {
        let query = assets::table
            .inner_join(room_assets::table.on(room_assets::asset_id.eq(assets::id)))
            .filter(room_assets::room_id.eq_any(room_ids))
            .select(assets::all_columns)
            .paginate_by(limit, page);

        let resources_with_total = query.load_and_count(conn).await?;

        Ok(resources_with_total)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_ids_and_size(conn: &mut DbConnection) -> Result<Vec<(AssetId, i64)>> {
        let query = assets::table
            .select((assets::id, assets::size))
            .order_by(assets::created_at.asc());

        let resources_with_total = query.load(conn).await?;

        Ok(resources_with_total)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_id(
        conn: &mut DbConnection,
        asset_id: AssetId,
        room_id: RoomId,
    ) -> Result<()> {
        conn.transaction(|conn| {
            async move {
                //FIXME: This check (as well as the room_id parameter) can be removed when assets have their own permission
                // check and don't rely on room permissions
                //
                // check if the asset exists for the specified room
                room_assets::table
                    .filter(
                        room_assets::asset_id
                            .eq(asset_id)
                            .and(room_assets::room_id.eq(room_id)),
                    )
                    .execute(conn)
                    .await?;

                diesel::delete(assets::table.filter(assets::id.eq(asset_id)))
                    .execute(conn)
                    .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await
    }

    /// Used for the internal deletion of assets
    ///
    /// When the request originates from a client, the [`Self::delete_by_id`] method should be used.
    #[tracing::instrument(err, skip_all)]
    pub async fn internal_delete_by_id(conn: &mut DbConnection, asset_id: &AssetId) -> Result<()> {
        let query = diesel::delete(assets::table.filter(assets::id.eq(asset_id)));

        query.execute(conn).await?;

        Ok(())
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_ids(conn: &mut DbConnection, asset_ids: &[AssetId]) -> Result<()> {
        let query = diesel::delete(assets::table.filter(assets::id.eq_any(asset_ids)));

        query.execute(conn).await?;

        Ok(())
    }
}

#[derive(Debug, Insertable)]
pub struct RoomAsset {
    pub room_id: RoomId,
    pub asset_id: AssetId,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = assets)]
pub struct NewAsset {
    pub id: AssetId,
    pub namespace: Option<ModuleId>,
    pub kind: String,
    pub filename: String,
    pub tenant_id: TenantId,
    pub size: i64,
}

impl NewAsset {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert_for_room(self, conn: &mut DbConnection, room_id: RoomId) -> Result<Asset> {
        conn.transaction(|conn| {
            async move {
                let asset: Asset = self.insert_into(assets::table).get_result(conn).await?;

                RoomAsset {
                    room_id,
                    asset_id: asset.id,
                }
                .insert_into(room_assets::table)
                .execute(conn)
                .await?;

                Ok(asset)
            }
            .scope_boxed()
        })
        .await
    }
}

pub type AssetRoomIdEventIdTuple = (Asset, RoomId, Option<EventId>);

#[tracing::instrument(err, skip_all)]
pub async fn get_all_for_room_owner_paginated_ordered(
    conn: &mut DbConnection,
    user_id: UserId,
    limit: i64,
    page: i64,
    sorting: AssetSorting,
    order: Ordering,
) -> Result<(Vec<AssetRoomIdEventIdTuple>, i64)> {
    let mut query = room_assets::table
        .inner_join(assets::table)
        .inner_join(rooms::table.left_join(events::table))
        .filter(rooms::created_by.eq(user_id))
        .select((
            assets::all_columns,
            rooms::columns::id,
            events::columns::id.nullable(),
        ))
        .into_boxed();

    // There was no sane approach to move this block to it's own function or call asc/desc on a
    // generalized column.
    query = match (order, sorting) {
        (Ordering::Ascending, AssetSorting::Filename) => query.order(assets::filename.asc()),
        (Ordering::Ascending, AssetSorting::Size) => query.order(assets::size.asc()),
        (Ordering::Ascending, AssetSorting::Namespace) => query.order(assets::namespace.asc()),
        (Ordering::Ascending, AssetSorting::Kind) => query.order(assets::kind.asc()),
        (Ordering::Ascending, AssetSorting::CreatedAt) => query.order(assets::created_at.asc()),

        (Ordering::Descending, AssetSorting::Filename) => query.order(assets::filename.desc()),
        (Ordering::Descending, AssetSorting::Size) => query.order(assets::size.desc()),
        (Ordering::Descending, AssetSorting::Namespace) => query.order(assets::namespace.desc()),
        (Ordering::Descending, AssetSorting::Kind) => query.order(assets::kind.desc()),
        (Ordering::Descending, AssetSorting::CreatedAt) => query.order(assets::created_at.desc()),
    };

    let query = query.paginate_by(limit, page);

    Ok(query.load_and_count(conn).await?)
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = assets)]
pub struct UpdateAsset {
    pub size: Option<i64>,
    pub filename: Option<String>,
}

impl UpdateAsset {
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(self, conn: &mut DbConnection, asset_id: AssetId) -> Result<Asset> {
        let target = assets::table.filter(assets::id.eq(&asset_id));
        let asset = diesel::update(target).set(self).get_result(conn).await?;

        Ok(asset)
    }
}
