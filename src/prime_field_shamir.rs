use std::iter::{Product, Sum};
use std::ops::{Add, Mul};
use ark_ff::PrimeField;

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

    fn evaluate(&self, x: F) -> F {
        self.coef.iter().rev().cloned().reduce(|acc, curr| acc * x + curr).unwrap()
    }
    // func to generate the polynomial using >= k no of points
    fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
        xs.iter().zip(ys.iter())
            .map(|(x, y)| Self::basis(x, &xs.clone()).scalar_mul(y.clone()))
            .sum()
    }

    fn basis(x: &F, interpolating_set: &[F]) -> Self {
        // numerator
        let numerator: Self = interpolating_set.iter().filter(|x_val| *x_val != x).map(|x_in_set| Univariatepoly::new(vec![x_in_set.neg(), F::one()])).product();
    
        // denominator
        let denominator = F::one() / numerator.evaluate(*x);
        numerator.scalar_mul(denominator)
    }

    fn scalar_mul(&self, scalar: F) -> Self {
        let new_coef = self.coef.iter().map(|coef| *coef * scalar).collect();
        Univariatepoly {
            coef: new_coef
        }
    }
}

impl <F: PrimeField> Add for &Univariatepoly<F> {
    type Output = Univariatepoly<F>;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.degree() < rhs.degree() {
            (rhs.clone(), self)
        } else {
            (self.clone(), rhs)
        };

        let _ = bigger.coef.iter_mut().zip(smaller.coef.iter()).map(|(big, sm)| *big += sm).collect::<()>();

        Univariatepoly::new(bigger.coef)
    }
}

impl <F: PrimeField> Sum for Univariatepoly<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut new_sum = Univariatepoly::new(vec![F::zero()]);
        for poly in iter {
            new_sum = &new_sum + &poly 
        };
        new_sum
    } 
}

impl <F: PrimeField>Mul for &Univariatepoly<F> {
    type Output = Univariatepoly<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![F::zero(); new_degree + 1];
        for i in 0..self.coef.len() {
            for j in 0..rhs.coef.len() {
                result[i + j] += self.coef[i] * rhs.coef[j]
            }
        };
        Univariatepoly {
            coef: result
        }
    }
}

impl <F: PrimeField> Product for Univariatepoly<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Univariatepoly::new(vec![F::one()]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

// func to generate random coefficients for the polynomial
fn generate_random_coef<F: PrimeField>() -> F {
    let mut rng = rand::thread_rng();
    F::rand(&mut rng)
}

// func to construct univariate poly from  random coef given threshold
fn construct_polynomial<F: PrimeField>(secret: F, k: u32) -> Univariatepoly<F> {
    let mut coef = Vec::new();
    coef.push(secret);
    for _ in 1..k {
        let new_coef = generate_random_coef();
        coef.push(new_coef);
    }
    Univariatepoly::new(coef)
}

// the polynomial constructed above would be evaluated at chosen points equal to n, where n must be 
// greater than k. When combined with the x values gives us the interpolating set
fn share_secret_points<F: PrimeField>(secret: F, n_shares: u32, k: u32) -> (Vec<F>, Vec<F>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let poly = construct_polynomial(secret, k);
    for i in 1..=n_shares {
        let eval_value = poly.evaluate(i.into());
        xs.push(i.into());
        ys.push(eval_value);
    }
    (xs, ys)
}

fn main() {

}

#[cfg(test)]
mod tests {
    use crate::Univariatepoly;
    use crate::share_secret_points;
    use ark_bn254::Fq;

    fn poly_1() -> Univariatepoly<Fq> {
        // f(x) = 1 + 2x + 3x^2
        Univariatepoly {
            coef: vec![Fq::from(1), Fq::from(2), Fq::from(3)]
        }
    }

    fn poly_2() -> Univariatepoly<Fq> {
        // f(x) = 4x + 3 + 5x^11
        Univariatepoly {
            coef: [vec![Fq::from(3), Fq::from(4)], vec![Fq::from(0); 9], vec![Fq::from(5)]].concat(),
        }
    }

    #[test]
    fn test_degree() {
        assert_eq!(poly_1().degree(), 2);
    }

    #[test]
    fn test_evaluate() {
        // f(2) = 1 + 2(2) + 3(2^2) = 1 + 4 + 12 = 17
        assert_eq!(poly_1().evaluate(Fq::from(2)), Fq::from(17));
    }

    #[test]
    fn test_addition() {
        assert_eq!((&poly_1() + &poly_2()).coef, [vec![Fq::from(4), Fq::from(6), Fq::from(3)], vec![Fq::from(0); 8], vec![Fq::from(5)]].concat())
    }

    #[test]
    fn test_mul() {
        // f(x) = 5 + 2x^2
        let poly_1 = Univariatepoly {
            coef: vec![Fq::from(5), Fq::from(0), Fq::from(2)],
        };
        // f(x) = 2x + 6
        let poly_2 = Univariatepoly {
            coef: vec![Fq::from(6), Fq::from(2)],
        };

        assert_eq!((&poly_1 * &poly_2).coef, vec![Fq::from(30), Fq::from(10), Fq::from(12), Fq::from(4)]);
    }

    #[test]
    fn test_interpolate() {
        let ans = Univariatepoly::interpolate(vec![Fq::from(2), Fq::from(4)], vec![Fq::from(4), Fq::from(8)]);

        assert_eq!(ans.coef, vec![Fq::from(0), Fq::from(2)]);
    }

    #[test]
    fn test_secret_sharing_and_recovery() {
        let (xs, ys) = share_secret_points(Fq::from(8), 10, 3);
        let x_interpol_set = &xs[0..3];
        let y_interpol_set = &ys[0..3];

        let ans = Univariatepoly::interpolate(x_interpol_set.to_vec(), y_interpol_set.to_vec());

        assert_eq!(ans.evaluate(Fq::from(0)), Fq::from(8));
    }
}
