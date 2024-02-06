#[macro_export]
macro_rules! get_apollo_instance {
    ($chain_id:expr) => {{
        use apollo_utils::nat::ToNativeTypes;
        let apollo_instance_result = crate::STATE.with(|s| {
            s.borrow()
                .chains
                .get(&$chain_id.to_u32())
                .ok_or(apollo_utils::errors::ApolloError::ChainNotFound($chain_id))
        });

        apollo_instance_result?.0
    }};
}

#[macro_export]
macro_rules! update_apollo_instance {
    ($chain_id:expr, $apollo_instance:expr) => {{
        use apollo_utils::nat::ToNativeTypes;
        let result = crate::STATE.with(|s| {
            s.borrow_mut()
                .chains
                .insert($chain_id.to_u32(), Cbor($apollo_instance))
                .ok_or(apollo_utils::errors::ApolloError::ChainNotFound($chain_id))
        });

        result?
    }};
}
