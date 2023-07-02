use ec_generic::{EllipticCurve, FiniteField, Point};
use num_bigint::{BigUint, RandBigInt};
use rand;
use sha256::digest;

pub struct ECDSA {
    elliptic_curve: EllipticCurve,
    a_gen: Point,
    q_order: BigUint,
}

impl ECDSA {
    // Generates: d, B where B = d A
    pub fn generate_key_pair(&self) -> (BigUint, Point) {
        let priv_key = self.generate_priv_key();
        let pub_key = self.generate_pub_key(&priv_key);
        (priv_key, pub_key)
    }

    pub fn generate_priv_key(&self) -> BigUint {
        self.generate_random_positive_number_less_than(&self.q_order)
    }

    pub fn generate_pub_key(&self, priv_key: &BigUint) -> Point {
        self.elliptic_curve.scalar_mul(&self.a_gen, &priv_key)
    }

    // (0, max)
    pub fn generate_random_positive_number_less_than(&self, max: &BigUint) -> BigUint {
        let mut rng = rand::thread_rng();
        rng.gen_biguint_range(&BigUint::from(1u32), &max)
    }

    ///
    /// R = k A -> take `r = x` component
    /// s = (hash(message) + d * r) * k^(-1) mod q
    ///
    pub fn sign(
        &self,
        hash: &BigUint,
        priv_key: &BigUint,
        k_random: &BigUint,
    ) -> (BigUint, BigUint) {
        assert!(
            *hash < self.q_order,
            "Hash is bigger than the order of the EC group"
        );
        assert!(
            *priv_key < self.q_order,
            "Private key is bigger than the order of the EC group"
        );
        assert!(
            *k_random < self.q_order,
            "Random number `k` is bigger than the order of the EC group"
        );

        let r_point = self.elliptic_curve.scalar_mul(&self.a_gen, k_random);

        if let Point::Coor(r, _) = r_point {
            let s = FiniteField::mult(&r, priv_key, &self.q_order);
            let s = FiniteField::add(&s, hash, &self.q_order);
            let k_inv = FiniteField::inv_mult_prime(k_random, &self.q_order);
            let s = FiniteField::mult(&s, &k_inv, &self.q_order);

            return (r, s);
        }

        panic!("The random point R should not be the identity");
    }

    ///
    /// u1 = s^(-1) * hash(message) mod q
    /// u2 = s^(-1) * r mod q
    /// P = u1 A + u2 B mod q = (xp, yp)
    /// if r == xp then verified!
    ///
    pub fn verify(&self, hash: &BigUint, pub_key: &Point, signature: &(BigUint, BigUint)) -> bool {
        assert!(
            *hash < self.q_order,
            "Hash is bigger than the order of the EC group"
        );

        let (r, s) = signature;
        let s_inv = FiniteField::inv_mult_prime(&s, &self.q_order);
        let u1 = FiniteField::mult(&s_inv, hash, &self.q_order);
        let u2 = FiniteField::mult(&s_inv, &r, &self.q_order);
        let u1a = self.elliptic_curve.scalar_mul(&self.a_gen, &u1);
        let u2b = self.elliptic_curve.scalar_mul(&pub_key, &u2);
        let p = self.elliptic_curve.add(&u1a, &u2b);

        if let Point::Coor(xp, _) = p {
            return xp == *r;
        }

        panic!("Point P = u1 A + u2 B cannot be the identity");
    }

    /// 0 < hash < max
    pub fn generate_hash_less_than(&self, message: &str, max: &BigUint) -> BigUint {
        let digest = digest(message);
        let hash_bytes = hex::decode(&digest).expect("Could not convert hash to Vec<u8>");
        let hash = BigUint::from_bytes_be(&hash_bytes);
        let hash = hash.modpow(&BigUint::from(1u32), &(max - BigUint::from(1u32)));
        let hash = hash + BigUint::from(1u32);
        hash
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sign_verify() {
        let elliptic_curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };

        let a_gen = Point::Coor(BigUint::from(5u32), BigUint::from(1u32));

        let q_order = BigUint::from(19u32);

        let ecdsa = ECDSA {
            elliptic_curve,
            a_gen,
            q_order,
        };

        let priv_key = BigUint::from(7u32);
        let pub_key = ecdsa.generate_pub_key(&priv_key);

        let k_random = BigUint::from(18u32);

        let message = "Bob -> 1 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let signature = ecdsa.sign(&hash, &priv_key, &k_random);

        let verify_result = ecdsa.verify(&hash, &pub_key, &signature);

        assert!(verify_result, "Verification should success");
    }

    #[test]
    fn test_sign_verify_tempered_message() {
        let elliptic_curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };

        let a_gen = Point::Coor(BigUint::from(5u32), BigUint::from(1u32));

        let q_order = BigUint::from(19u32);

        let ecdsa = ECDSA {
            elliptic_curve,
            a_gen,
            q_order,
        };

        let priv_key = BigUint::from(7u32);
        let pub_key = ecdsa.generate_pub_key(&priv_key);

        let k_random = BigUint::from(18u32);

        let message = "Bob -> 1 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let signature = ecdsa.sign(&hash, &priv_key, &k_random);

        let message = "Bob -> 2 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let verify_result = ecdsa.verify(&hash, &pub_key, &signature);

        assert!(
            !verify_result,
            "Verification should fail when message is tempered"
        );
    }

    #[test]
    fn test_sign_verify_tempered_signature() {
        let elliptic_curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };

        let a_gen = Point::Coor(BigUint::from(5u32), BigUint::from(1u32));

        let q_order = BigUint::from(19u32);

        let ecdsa = ECDSA {
            elliptic_curve,
            a_gen,
            q_order,
        };

        let priv_key = BigUint::from(7u32);
        let pub_key = ecdsa.generate_pub_key(&priv_key);

        let k_random = BigUint::from(13u32);

        let message = "Bob -> 1 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let signature = ecdsa.sign(&hash, &priv_key, &k_random);
        let (r, s) = signature;
        let tempered_signature = (
            (r + BigUint::from(1u32)).modpow(&BigUint::from(1u32), &ecdsa.q_order),
            s,
        );

        let verify_result = ecdsa.verify(&hash, &pub_key, &tempered_signature);

        assert!(
            !verify_result,
            "Verification should fail when signature is tempered"
        );
    }

    #[test]
    fn test_secp256_sign_verify() {
        let p = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
            16,
        )
        .expect("could not convert p");

        let q_order = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .expect("could not convert n");

        let gx = BigUint::parse_bytes(
            b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            16,
        )
        .expect("could not convert gx");

        let gy = BigUint::parse_bytes(
            b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            16,
        )
        .expect("could not convert gy");

        let elliptic_curve = EllipticCurve {
            a: BigUint::from(0u32),
            b: BigUint::from(7u32),
            p,
        };

        let a_gen = Point::Coor(gx, gy);

        let ecdsa = ECDSA {
            elliptic_curve,
            a_gen,
            q_order,
        };

        let priv_key = BigUint::parse_bytes(
            b"483ADB7726A3C4655DA4FBFC0E1208A8F017B448A68554199C47D08FFB10E4B9",
            16,
        )
        .expect("Could not convert hex to private key");

        let pub_key = ecdsa.generate_pub_key(&priv_key);

        let k_random = BigUint::parse_bytes(
            b"19BE666EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B15E81798",
            16,
        )
        .expect("Could not convert hex to private key");

        let message = "Bob -> 1 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let signature = ecdsa.sign(&hash, &priv_key, &k_random);

        let verify_result = ecdsa.verify(&hash, &pub_key, &signature);

        assert!(verify_result, "Verification should have succeed");
    }

    #[test]
    fn test_secp256_sign_verify_tempered_message() {
        let p = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
            16,
        )
        .expect("could not convert p");

        let q_order = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .expect("could not convert n");

        let gx = BigUint::parse_bytes(
            b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            16,
        )
        .expect("could not convert gx");

        let gy = BigUint::parse_bytes(
            b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            16,
        )
        .expect("could not convert gy");

        let elliptic_curve = EllipticCurve {
            a: BigUint::from(0u32),
            b: BigUint::from(7u32),
            p,
        };

        let a_gen = Point::Coor(gx, gy);

        let ecdsa = ECDSA {
            elliptic_curve,
            a_gen,
            q_order,
        };

        let priv_key = BigUint::parse_bytes(
            b"483ADB7726A3C4655DA4FBFC0E1208A8F017B448A68554199C47D08FFB10E4B9",
            16,
        )
        .expect("Could not convert hex to private key");

        let pub_key = ecdsa.generate_pub_key(&priv_key);

        let k_random = BigUint::parse_bytes(
            b"19BE666EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B15E81798",
            16,
        )
        .expect("Could not convert hex to private key");

        let message = "Bob -> 1 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let signature = ecdsa.sign(&hash, &priv_key, &k_random);

        let message = "Bob -> 2 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let verify_result = ecdsa.verify(&hash, &pub_key, &signature);

        assert!(
            !verify_result,
            "Verification should have failed due to tempered message"
        );
    }

    #[test]
    fn test_secp256_sign_verify_tempered_signature() {
        let p = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
            16,
        )
        .expect("could not convert p");

        let q_order = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .expect("could not convert n");

        let gx = BigUint::parse_bytes(
            b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            16,
        )
        .expect("could not convert gx");

        let gy = BigUint::parse_bytes(
            b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            16,
        )
        .expect("could not convert gy");

        let elliptic_curve = EllipticCurve {
            a: BigUint::from(0u32),
            b: BigUint::from(7u32),
            p,
        };

        let a_gen = Point::Coor(gx, gy);

        let ecdsa = ECDSA {
            elliptic_curve,
            a_gen,
            q_order,
        };

        let priv_key = BigUint::parse_bytes(
            b"483ADB7726A3C4655DA4FBFC0E1208A8F017B448A68554199C47D08FFB10E4B9",
            16,
        )
        .expect("Could not convert hex to private key");

        let pub_key = ecdsa.generate_pub_key(&priv_key);

        let k_random = BigUint::parse_bytes(
            b"19BE666EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B15E81798",
            16,
        )
        .expect("Could not convert hex to private key");

        let message = "Bob -> 1 BTC -> Alice";
        let hash = ecdsa.generate_hash_less_than(message, &ecdsa.q_order);

        let signature = ecdsa.sign(&hash, &priv_key, &k_random);
        let (r, s) = signature;
        let tempered_signature = (
            (r + BigUint::from(1u32)).modpow(&BigUint::from(1u32), &ecdsa.q_order),
            s,
        );

        let verify_result = ecdsa.verify(&hash, &pub_key, &tempered_signature);

        assert!(
            !verify_result,
            "Verification should have failed due to tempered signature"
        );
    }
}
