use num_bigint::{BigUint, ToBigUint};
use num_traits::{One, Zero};
use rand::Rng;
pub mod class_impl; 
// mod prime_field_shamir;
use rand::RngCore;
use std::iter::{Product, Sum};
use std::ops::{Add, Mul};
use lazy_static::lazy_static;
use num_traits::ToPrimitive;
use ark_ff::PrimeField;
use ark_bn254::Fq;

// function to get the prime p within which random coefs can be gotten
    // function to generate random coeficients
    // function to construct the polynomial i.e return a Univariatepoly struct based on the chosen 
    // threshold after the coefs have been rnadonmly generated
    // function to evaluate the polynomial at different points the number of times specified by the number 
    // of persons you want to share the secret with.
    // function to interpolate and and generate back the polynomial given k no of points or more

    // Assumptions:
    // 1. D is a scalar value and is equal to 50. I dont know if should make it a global var or pass it 
    // into the functions
    // 2. Threshold, k is 4 --> polynomial degree is k - 1 = 3
    // 3. The secret is to be shared with 10 persons --> n = 10
lazy_static! {
    static ref PRIME: BigUint = {
        let base = 2_u32.to_biguint().unwrap();
        let exp: u32 = 256;
        let minus_term = 189_u32.to_biguint().unwrap();
        base.pow(exp) - minus_term
    };
}

#[derive(Debug, PartialEq, Clone)]
struct Univariatepoly<F: PrimeField> {
    coef: Vec<F>
}

impl <F: PrimeField> Univariatepoly<F> {
    fn new(coef: Vec<F>) -> Self {
        Univariatepoly { coef }
    }

    fn degree(&self) -> usize {
        self.coef.len() - 1
    }

    fn evaluate(&self, x: u32) -> BigUint {
        let x_big = x.to_biguint().unwrap();
        if self.coef.is_empty() {
            return BigUint::zero();
        }
        self.coef.iter().rev().cloned().reduce(|acc, curr| acc * &x_big + curr).unwrap() % &*PRIME
    }

    fn mod_scalar_mul(&self, scalar: BigUint) -> Self {
        let new_coef = self.coef.iter()
            .map(|coef| (coef * &scalar) % &*PRIME)
            .collect();
        Univariatepoly { coef: new_coef }
    }

    fn basis(x: &BigUint, interpolating_set: Vec<BigUint>) -> Self {
        // numerator
        let numerator: Univariatepoly = interpolating_set.iter()
            .filter(|x_val| *x_val != x)
            .map(|x_in_set| {
                let neg_x = (&*PRIME - x_in_set) % &*PRIME;
                Univariatepoly::new(vec![neg_x, BigUint::one()])
            })
            .product();
    
        // denominator
        let denominator = BigUint::one() / numerator.evaluate(x.to_u32().unwrap());
        numerator.mod_scalar_mul(denominator)
    }
}

impl Product for Univariatepoly {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Univariatepoly::new(vec![BigUint::one()]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

impl Sum for Univariatepoly {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Univariatepoly::new(vec![BigUint::zero()]);
        for poly in iter {
            result = &result + &poly;
        }
        result
    }
}

impl Add for &Univariatepoly {
    type Output = Univariatepoly;
    
    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.degree() < rhs.degree() {
            (rhs.clone(), self)
        } else {
            (self.clone(), rhs)
        };
    
        for (big, sm) in bigger.coef.iter_mut().zip(smaller.coef.iter()) {
            *big += sm;
            *big %= &*PRIME;
        }
    
        Univariatepoly::new(bigger.coef)
    }
}

impl Mul for &Univariatepoly {
    type Output = Univariatepoly;
    
    fn mul(self, rhs: Self) -> Self::Output {
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![BigUint::zero(); new_degree + 1];
        
        for i in 0..self.coef.len() {
            for j in 0..rhs.coef.len() {
                let prod = (&self.coef[i] * &rhs.coef[j]) % &*PRIME;
                result[i + j] = (&result[i + j] + &prod) % &*PRIME;
            }
        }
        
        Univariatepoly::new(result)
    }
}

impl Add for Univariatepoly {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl Mul for Univariatepoly {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

fn get_p() -> BigUint {
    PRIME.clone()
}

// func to generate random coefficients for the polynomial
fn generate_random_coef(p: &BigUint) -> BigUint {
    let mut random_no = rand::thread_rng();
    let bytes_of_p = (p.bits() as usize + 7) / 8;
    loop {
        let mut buffer = vec![0_u8; bytes_of_p];
        random_no.try_fill_bytes(&mut buffer);
        let num = BigUint::from_bytes_be(&buffer);
        if num < *p {
            return num;
        }
    }
}

// func to construct univariate poly from  random coef given threshold
fn construct_polynomial(secret: BigUint, k: u32) -> Univariatepoly {
    let p = get_p();
    let mut coef = Vec::new();
    coef.push(secret);
    for _ in 1..k {
        let new_coef = generate_random_coef(&p);
        coef.push(new_coef);
    }
    Univariatepoly::new(coef)
}

// the polynomial constructed above would be evaluated at chosen points equal to n, where n must be 
    // greater than k. When combined with the x values gives us the interpolating set
fn share_secret_points(secret: BigUint, n_shares: u32, k: u32) -> (Vec<BigUint>, Vec<BigUint>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let poly = construct_polynomial(secret, k);
    for i in 1..=n_shares {
        let eval_value = poly.evaluate(i);
        xs.push(i.to_biguint().unwrap());
        ys.push(eval_value);
    }
    (xs, ys)
}

// func to generate the polynomial with >=k no of points
fn interpolate(xs: Vec<BigUint>, ys: Vec<BigUint>) -> Univariatepoly {
    xs.iter().zip(ys.iter())
        .map(|(x, y)| Univariatepoly::basis(x, xs.clone()).mod_scalar_mul(y.clone()))
        .sum()
}

fn main() {
    // Example usage
    let secret = BigUint::from(42u32);
    let k = 3;  // threshold
    let n = 5;  // number of shares
    
    let (xs, ys) = share_secret_points(secret.clone(), n, k);
    println!("Shares generated: {:?}", xs.iter().zip(ys.iter()).collect::<Vec<_>>());
    println!("xs: {:?}", xs[0..k as usize].to_vec());
    println!("ys: {:?}", ys[0..k as usize].to_vec());
    
    let recovered = interpolate(xs, ys);
    println!("Recovered secret: {:?}", recovered.coef[0]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_evaluation() {
        let coef = vec![
            BigUint::from(1u32),  // constant term
            BigUint::from(2u32),  // coefficient of x
            BigUint::from(3u32),  // coefficient of x^2
        ];
        let poly = Univariatepoly::new(coef);
        
        // For x = 2, should compute 1 + 2(2) + 3(2^2) = 1 + 4 + 12 = 17
        assert_eq!(poly.evaluate(2), BigUint::from(17u32));
    }

    #[test]
    fn test_polynomial_addition() {
        let poly1 = Univariatepoly::new(vec![
            BigUint::from(1u32),
            BigUint::from(2u32),
        ]);
        let poly2 = Univariatepoly::new(vec![
            BigUint::from(3u32),
            BigUint::from(4u32),
        ]);
        
        let result = &poly1 + &poly2;
        assert_eq!(result.coef, vec![
            BigUint::from(4u32),  // 1 + 3
            BigUint::from(6u32),  // 2 + 4
        ]);
    }

    #[test]
    fn test_polynomial_multiplication() {
        let poly1 = Univariatepoly::new(vec![
            BigUint::from(1u32),  // 1 + 2x
            BigUint::from(2u32),
        ]);
        let poly2 = Univariatepoly::new(vec![
            BigUint::from(3u32),  // 3 + 4x
            BigUint::from(4u32),
        ]);
        
        let result = &poly1 * &poly2;
        assert_eq!(result.coef, vec![
            BigUint::from(3u32),   // constant term: 1 * 3
            BigUint::from(10u32),  // coefficient of x: 1 * 4 + 2 * 3
            BigUint::from(8u32),   // coefficient of x^2: 2 * 4
        ]);
    }

    #[test]
    fn test_secret_sharing_and_recovery() {
        let secret = BigUint::from(42u32);
        let k = 3;  // threshold
        let n = 5;  // number of shares
        
        // Generate shares
        let (xs, ys) = share_secret_points(secret.clone(), n, k);
        println!("xs: {:?}", xs[0..k as usize].to_vec());
        println!("ys: {:?}", ys[0..k as usize].to_vec());
        
        // Take any k shares and recover the secret
        let recovered_poly = interpolate(
            xs[0..k as usize].to_vec(),
            ys[0..k as usize].to_vec()
        );
        
        // The constant term should be our secret
        assert_eq!(recovered_poly.coef[0], secret);
    }

    #[test]
    fn test_random_coefficient_generation() {
        let p = get_p();
        let coef = generate_random_coef(&p);
        assert!(coef < p);
    }

    #[test]
    fn test_polynomial_scalar_multiplication() {
        let poly = Univariatepoly::new(vec![
            BigUint::from(2u32),
            BigUint::from(3u32),
        ]);
        let scalar = BigUint::from(2u32);
        
        let result = poly.mod_scalar_mul(scalar);
        assert_eq!(result.coef, vec![
            BigUint::from(4u32),  // 2 * 2
            BigUint::from(6u32),  // 3 * 2
        ]);
    }

    // #[test]
    // fn test_polynomial_interpolation() {
    //     let ans = Univariatepoly::interpolate(vec![2.0, 4.0], vec![4.0, 8.0]);
    // }
}