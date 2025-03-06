use scrypto::prelude::*;

#[blueprint]
mod mockocipool {
    struct MockOciPool {
        vault_a: Vault,
        vault_b: Vault,
    }

    impl MockOciPool {
        pub fn instantiate(
            bucket_a: Bucket,
            bucket_b: Bucket,
        ) -> (Global<MockOciPool>, ComponentAddress) {
            let vault_a = Vault::with_bucket(bucket_a);
            let vault_b = Vault::with_bucket(bucket_b);

            let component = Self { vault_a, vault_b }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();

            let component_address = component.address();

            (component, component_address)
        }

        pub fn swap(&mut self, bucket_a: Bucket) -> Bucket {
            let return_bucket = self.vault_b.take(bucket_a.amount());
            self.vault_a.put(bucket_a);

            return_bucket
        }
    }
}
