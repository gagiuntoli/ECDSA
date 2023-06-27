use std::iter::Product;

use num_bigint::BigUint;

#[derive(PartialEq, Clone, Debug)]
enum Point {
    Coor(BigUint, BigUint),
    Identity,
}

#[derive(PartialEq, Clone, Debug)]
struct EllipticCurve {
    // y^2 = x^2 + a * x + b
    a: BigUint,
    b: BigUint,
    p: BigUint,
}

impl EllipticCurve {
    fn add(&self, c: &Point, d: &Point) -> Point {
        assert!(self.is_on_curve(c), "Point is not in curve");
        assert!(self.is_on_curve(d), "Point is not in curve");
        assert!(*c != *d, "Points should not be the same");

        match (c, d) {
            (Point::Identity, _) => d.clone(),
            (_, Point::Identity) => c.clone(),
            (Point::Coor(x1, y1), Point::Coor(x2, y2)) => {
                // s = (y2 - y1) / (x2 - x1) mod p
                // x3 = s^2 - x1 - x2  mod p
                // y3 = s(x1 - x3) - y1 mod p
                let y2minusy1 = FiniteField::subtract(y2, y1, &self.p);
                let x2minusx1 = FiniteField::subtract(x2, x1, &self.p);
                let s = FiniteField::divide(&y2minusy1, &x2minusx1, &self.p);
                let s2 = s.modpow(&BigUint::from(2u32), &self.p);
                let s2minusx1 = FiniteField::subtract(&s2, x1, &self.p);
                let x3 = FiniteField::subtract(&s2minusx1, x2, &self.p);
                let x1minusx3 = FiniteField::subtract(x1, &x3, &self.p);
                let sx1minusx3 = FiniteField::mult(&s, &x1minusx3, &self.p);
                let y3 = FiniteField::subtract(&sx1minusx3, &y1, &self.p);
                Point::Coor(x3, y3)
            }
        }
    }

    fn double(c: &Point) -> Point {
        todo!()
    }

    fn scalar_mul(c: &Point, d: &BigUint) -> Point {
        // addition/doubling algorithm
        // B = d * A
        todo!()
    }

    pub fn is_on_curve(&self, c: &Point) -> bool {
        match c {
            Point::Coor(x, y) => {
                // y^2 = x^3 + a * x + b
                let y2 = y.modpow(&BigUint::from(2u32), &self.p);
                let x3 = x.modpow(&BigUint::from(3u32), &self.p);
                let ax = FiniteField::mult(&self.a, x, &self.p);
                let x3plusax = FiniteField::add(&x3, &ax, &self.p);
                y2 == FiniteField::add(&x3plusax, &self.b, &self.p)
            }
            Point::Identity => true,
        }
    }
}

struct FiniteField {}

impl FiniteField {
    fn add(c: &BigUint, d: &BigUint, p: &BigUint) -> BigUint {
        // c + d = r mod p
        let r = c + d;
        r.modpow(&BigUint::from(1u32), p)
    }

    fn mult(c: &BigUint, d: &BigUint, p: &BigUint) -> BigUint {
        // c * d = r mod p
        let r = c * d;
        r.modpow(&BigUint::from(1u32), p)
    }

    fn inv_addition(c: &BigUint, p: &BigUint) -> BigUint {
        // -c mod p
        assert!(c < p, "c >= p");
        //format!("number: {} is bigger or equal than p: {}", c, p)
        p - c
    }

    fn subtract(c: &BigUint, d: &BigUint, p: &BigUint) -> BigUint {
        let d_inv = FiniteField::inv_addition(d, p);
        FiniteField::add(c, &d_inv, p)
    }

    fn inv_multiplication(c: &BigUint, p: &BigUint) -> BigUint {
        // TODO: this function uses Fermat's Little Theorem and thus it is only valid for a p prime
        // only for p prime
        // c^(-1) mod p = c^(p-2) mod p
        c.modpow(&(p - BigUint::from(2u32)), p)
    }

    fn divide(c: &BigUint, d: &BigUint, p: &BigUint) -> BigUint {
        let d_inv = FiniteField::inv_multiplication(d, p);
        FiniteField::mult(c, &d_inv, p)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_1() {
        let c = BigUint::from(4u32);
        let d = BigUint::from(10u32);
        let p = BigUint::from(11u32);

        let r = FiniteField::add(&c, &d, &p);

        assert_eq!(r, BigUint::from(3u32));
    }

    #[test]
    fn test_add_2() {
        let c = BigUint::from(4u32);
        let d = BigUint::from(10u32);
        let p = BigUint::from(31u32);

        let r = FiniteField::add(&c, &d, &p);

        assert_eq!(r, BigUint::from(14u32));
    }

    #[test]
    fn test_mul_1() {
        let c = BigUint::from(4u32);
        let d = BigUint::from(10u32);
        let p = BigUint::from(11u32);

        let r = FiniteField::mult(&c, &d, &p);

        assert_eq!(r, BigUint::from(7u32));
    }

    #[test]
    fn test_mul_2() {
        let c = BigUint::from(4u32);
        let d = BigUint::from(10u32);
        let p = BigUint::from(51u32);

        let r = FiniteField::mult(&c, &d, &p);

        assert_eq!(r, BigUint::from(40u32));
    }

    #[test]
    fn test_inv_addition_1() {
        let c = BigUint::from(4u32);
        let p = BigUint::from(51u32);

        let r = FiniteField::inv_addition(&c, &p);

        assert_eq!(r, BigUint::from(47u32));
    }

    #[test]
    #[should_panic]
    fn test_inv_addition_2() {
        let c = BigUint::from(52u32);
        let p = BigUint::from(51u32);

        FiniteField::inv_addition(&c, &p);
    }

    #[test]
    fn test_inv_addition_identity() {
        let c = BigUint::from(4u32);
        let p = BigUint::from(51u32);

        let c_inv = FiniteField::inv_addition(&c, &p);

        assert_eq!(c_inv, BigUint::from(47u32));
        assert_eq!(FiniteField::add(&c, &c_inv, &p), BigUint::from(0u32));
    }

    #[test]
    fn test_substract() {
        let c = BigUint::from(4u32);
        let p = BigUint::from(51u32);

        assert_eq!(FiniteField::subtract(&c, &c, &p), BigUint::from(0u32));
    }

    #[test]
    fn test_inv_multiplication_identity() {
        let c = BigUint::from(4u32);
        let p = BigUint::from(11u32);

        let c_inv = FiniteField::inv_multiplication(&c, &p);

        // 4 * 3 mod 11 = 12 mod 11 = 1
        assert_eq!(c_inv, BigUint::from(3u32));
        assert_eq!(FiniteField::mult(&c, &c_inv, &p), BigUint::from(1u32));
    }

    #[test]
    fn test_divide() {
        let c = BigUint::from(4u32);
        let p = BigUint::from(11u32);

        assert_eq!(FiniteField::divide(&c, &c, &p), BigUint::from(1u32));
    }

    #[test]
    fn test_ec_point_addition() {
        // y^2 = x^3 + 2x + 2 mod 17
        let ec = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };

        // (6,3) + (5,1) = (10,6)
        let p1 = Point::Coor(BigUint::from(6u32), BigUint::from(3u32));
        let p2 = Point::Coor(BigUint::from(5u32), BigUint::from(1u32));
        let pr = Point::Coor(BigUint::from(10u32), BigUint::from(6u32));

        let res = ec.add(&p1, &p2);
        assert_eq!(res, pr);

        let res = ec.add(&p2, &p1);
        assert_eq!(res, pr);
    }

    #[test]
    fn test_ec_point_addition_identity() {
        // y^2 = x^3 + 2x + 2 mod 17
        let ec = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };

        // (6,3) + (5,1) = (10,6)
        let p1 = Point::Coor(BigUint::from(6u32), BigUint::from(3u32));
        let p2 = Point::Identity;
        let pr = p1.clone();

        let res = ec.add(&p1, &p2);
        assert_eq!(res, pr);

        let res = ec.add(&p2, &p1);
        assert_eq!(res, pr);
    }
}
