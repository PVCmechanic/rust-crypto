use rug::{ops::Pow, Float, Integer, Rational};
use std::{
    fmt::Debug,
    io::{self, Write},
    ops::{AddAssign, Mul},
};

pub fn eval_poly(x: &Integer, f: &Vec<Integer>, n: &Integer) -> Integer {
    let mut sum = Integer::from(0);
    let mut x_power = Integer::from(1);
    for term in f {
        sum += Integer::from(&x_power * term);
        x_power *= x;
        if *n > 0 {
            sum %= n;
            x_power %= n;
        }
    }
    return sum;
}

fn eval_lattice_poly(x: &Integer, f: &Vec<Integer>, const_x: &Integer, n: &Integer) -> Integer {
    let mut sum = Integer::from(0);
    let mut x_power = Integer::from(1);
    let mut const_x_power = Integer::from(1);
    for term in f {
        sum += Integer::from(&x_power * term) / &const_x_power;
        x_power *= x;
        const_x_power *= const_x;
    }
    if *n > 0 {
        return sum % n;
    } else {
        return sum;
    }
}

fn eval_rational_lattice_poly(x: &Rational, f: &Vec<Integer>, const_x: &Integer) -> Rational {
    let mut sum = Rational::from(0);
    let mut x_power = Rational::from(1);
    let mut const_x_power = Integer::from(1);
    for term in f {
        sum += Rational::from(&x_power * term) / &const_x_power;
        x_power *= x;
        const_x_power *= const_x;
    }
    return sum;
}

pub fn multiply_poly<'a, T>(f: &'a [T], g: &'a [T]) -> Vec<T>
where
    &'a T: Mul<&'a T>,
    T: From<<&'a T as Mul<&'a T>>::Output>,
    T: From<i32>,
    T: AddAssign<T>,
    T: PartialEq<i32>,
    T: Debug,
{
    let debug = false;
    if debug {
        print!(" mult {:?}*{:?}", f, g);
    }
    let mut prod = Vec::new();
    for _ in 0..(degree(f) + degree(g) + 1) {
        prod.push(T::from(0));
    }

    for i in 0..(degree(f) as usize + 1) {
        for j in 0..(degree(g) as usize + 1) {
            prod[i + j] += T::from(&f[i] * &g[j]);
        }
    }
    if debug {
        println!("\t|\t   result {:?}", prod);
    }
    return prod;
}

fn degree<T>(f: &[T]) -> i32
where
    T: PartialEq<i32>,
{
    for (i, val) in f.iter().enumerate().rev() {
        if *val != 0 {
            return i as i32;
        }
    }
    return 0;
}

fn lead<T>(f: &[T]) -> (T, usize)
where
    T: PartialEq<i32>,
    T: From<i32>,
    T: Clone,
{
    for (i, val) in f.iter().enumerate().rev() {
        if *val != 0 {
            return (val.clone(), i);
        }
    }
    return (T::from(0), 0);
}

pub fn divide_poly(f: &Vec<Rational>, g: &Vec<Rational>) -> (Vec<Rational>, Vec<Rational>) {
    println!(" div {:?}/{:?}", f, g);
    assert!(degree(g) >= 0, "divide by 0");
    let g = &g[0..=(degree(g) as usize)];
    let mut q: Vec<Rational> = (0..f.len()).map(|_x| Rational::from(0)).collect();
    let mut r = f.clone();

    while r.iter().any(|x| *x != 0) && degree(&r) >= degree(g) {
        let (r_lead, r_power) = lead(&r);
        let (g_lead, g_power) = lead(&g);
        let t = r_lead / g_lead;
        let t_power = r_power - g_power;

        q[t_power] += &t;

        print!("  t={}, t_pow={} | ", t, t_power);
        for (i, val) in g.iter().enumerate() {
            r[i + t_power] -= Rational::from(val * &t);
            print!("r[{}]={},  ", i + t_power, r[i + t_power]);
        }
        println!("\n  q: {:?}\n  r: {:?}", q, r);
    }

    println!(" result q:{:?}, \tr:{:?}", q, r);
    return (q, r);
}

pub fn divide_poly_zn(f: &Vec<Integer>, g: &Vec<Integer>, n: &Integer) -> (Vec<Integer>, Vec<Integer>) {
    println!(" div {:?}/{:?}", f, g);
    assert!(degree(g) >= 0, "divide by 0");
    let g = &g[0..=(degree(g) as usize)];
    let mut q: Vec<Integer> = (0..f.len()).map(|_x| Integer::from(0)).collect();
    let mut r = f.clone();

    while r.iter().any(|x| *x != 0) && degree(&r) >= degree(g) {
        let (r_lead, r_power) = lead(&r);
        let (g_lead, g_power) = lead(&g);
        let g_inv = find_inverse(&g_lead, n);
        let t = r_lead * g_inv % n;
        
        let t_power = r_power - g_power;

        q[t_power] += &t;
        q[t_power] %= n;

        print!("  t={}, t_pow={} | ", t, t_power);
        for (i, val) in g.iter().enumerate() {
            r[i + t_power] -= Integer::from(val * &t);
            r[i + t_power] %= n;
            print!("r[{}]={},  ", i + t_power, r[i + t_power]);
        }
        println!("\n  q: {:?}\n  r: {:?}", q, r);
    }

    println!(" result q:{:?}, \tr:{:?}", q, r);
    return (q, r);

}

pub fn exp_poly(f: &Vec<Integer>, e: &Integer) -> Vec<Integer> {
    if *e == 0 {
        let mut result = Vec::new();
        result.push(Integer::from(1));
        for _ in 0..(f.len() - 1) {
            result.push(Integer::from(0));
        }
        return result;
    }
    let mut result = f.clone();

    for _ in 0..Integer::from(e - 1).to_i32().unwrap() {
        result = multiply_poly(&result, f);
    }
    return result;
}

pub fn gcd(a: &Integer, b: &Integer) -> Integer {
    let (r, _s, _t) = extended_euclidian(a, b);
    return r;
}

pub fn find_inverse(e: &Integer, n: &Integer) -> Integer {
    let (_r, _s, t) = extended_euclidian(n, e);

    if t < 0 {
        let t = t + n;
        return t;
    } else {
        return t;
    }
}

pub fn bezout(a: &Integer, b: &Integer) -> (Integer, Integer) {
    if a > b {
        let (_, s, t) = extended_euclidian(a, b);
        return (s, t);
    } else {
        let (_, s, t) = extended_euclidian(b, a);
        return (t, s);
    }
}

pub fn extended_euclidian(a: &Integer, b: &Integer) -> (Integer, Integer, Integer) {
    let a = Integer::from(a);
    let b = Integer::from(b);
    let mut qs = Vec::new();
    let mut rs = Vec::new();
    let mut ss = Vec::new();
    let mut ts = Vec::new();

    rs.push(a);
    rs.push(b);

    ss.push(Integer::from(1));
    ss.push(Integer::from(0));

    ts.push(Integer::from(0));
    ts.push(Integer::from(1));

    // println!("r:{:x} \ts:{:x} \tt:{:x}", rs[0], ss[0], ts[0]);
    // println!("r:{:x} \ts:{:x} \tt:{:x}", rs[1], ss[1], ts[1]);

    let mut step = 1;
    while *rs.last().unwrap() != 0 {
        let new_q = Integer::from(&rs[step - 1] / &rs[step]);
        let new_r = Integer::from(&rs[step - 1] % &rs[step]);
        let new_s = &ss[step - 1] - Integer::from(&new_q * &ss[step]);
        let new_t = &ts[step - 1] - Integer::from(&new_q * &ts[step]);

        // println!("r:{:x} \ts:{:x} \tt:{:x}", new_r, new_s, new_t);
        qs.push(new_q);
        rs.push(new_r);
        ss.push(new_s);
        ts.push(new_t);

        step += 1;
    }

    // println!();
    rs.pop();
    ss.pop();
    ts.pop();
    return (rs.pop().unwrap(), ss.pop().unwrap(), ts.pop().unwrap());
}

pub fn poly_extended_euclidian_zn(
    a: &Vec<Integer>,
    b: &Vec<Integer>,
    n: &Integer
) -> (Vec<Integer>, Vec<Integer>, Vec<Integer>) {
    let debug = true;

    let a = a.clone();
    let b = b.clone();
    let mut qs = Vec::new();
    let mut rs = Vec::new();
    let mut ss = Vec::new();
    let mut ts = Vec::new();

    rs.push(a);
    rs.push(b);

    let mut identity_func = Vec::new();
    identity_func.push(Integer::from(1));
    ss.push(identity_func);
    let mut zero_func = Vec::new();
    zero_func.push(Integer::from(0));
    ss.push(zero_func);

    let mut zero_func = Vec::new();
    zero_func.push(Integer::from(0));
    ts.push(zero_func);
    let mut identity_func = Vec::new();
    identity_func.push(Integer::from(1));
    ts.push(identity_func);

    if debug {
        println!("r:{:?} \ts:{:?} \tt:{:?}", rs[0], ss[0], ts[0]);
        println!("r:{:?} \ts:{:?} \tt:{:?}", rs[1], ss[1], ts[1]);
    }

    let mut step = 1;
    while rs.last().unwrap().iter().any(|x| *x != 0) {
        let (new_q, new_r) = divide_poly_zn(&rs[step - 1], &rs[step], n);

        let q_times_s = multiply_poly(&new_q, &ss[step]);
        let mut new_s = ss[step - 1].clone();
        for _ in 0..(q_times_s.len() - new_s.len()) {
            new_s.push(Integer::from(0));
        }
        for (elem_s, elem_q) in new_s.iter_mut().zip(q_times_s) {
            *elem_s -= elem_q;
            *elem_s %= n;
        }

        let q_times_t = multiply_poly(&new_q, &ts[step]);
        let mut new_t = ts[step - 1].clone();
        for _ in 0..(q_times_t.len() - new_t.len()) {
            new_t.push(Integer::from(0));
        }
        for (elem_t, elem_q) in new_t.iter_mut().zip(q_times_t) {
            *elem_t -= elem_q;
            *elem_t %= n;
        }

        if debug {
            println!("r:{:?} \ts:{:?} \tt:{:?}", new_r, new_s, new_t);
        }
        qs.push(new_q);
        rs.push(new_r);
        ss.push(new_s);
        ts.push(new_t);

        step += 1;
    }

    if debug {
        println!();
    }
    rs.pop();
    ss.pop();
    ts.pop();
    return (rs.pop().unwrap(), ss.pop().unwrap(), ts.pop().unwrap());
}

pub fn poly_extended_euclidian(
    a: &Vec<Integer>,
    b: &Vec<Integer>,
) -> (Vec<Rational>, Vec<Rational>, Vec<Rational>) {
    let debug = true;

    let a: Vec<Rational> = a.iter().map(|x| Rational::from(x)).collect();
    let b = b.iter().map(|x| Rational::from(x)).collect();
    let mut qs = Vec::new();
    let mut rs = Vec::new();
    let mut ss = Vec::new();
    let mut ts = Vec::new();

    rs.push(a);
    rs.push(b);

    let mut identity_func = Vec::new();
    identity_func.push(Rational::from(1));
    ss.push(identity_func);
    let mut zero_func = Vec::new();
    zero_func.push(Rational::from(0));
    ss.push(zero_func);

    let mut zero_func = Vec::new();
    zero_func.push(Rational::from(0));
    ts.push(zero_func);
    let mut identity_func = Vec::new();
    identity_func.push(Rational::from(1));
    ts.push(identity_func);

    if debug {
        println!("r:{:?} \ts:{:?} \tt:{:?}", rs[0], ss[0], ts[0]);
        println!("r:{:?} \ts:{:?} \tt:{:?}", rs[1], ss[1], ts[1]);
    }

    let mut step = 1;
    while rs.last().unwrap().iter().any(|x| *x != 0) {
        let (new_q, new_r) = divide_poly(&rs[step - 1], &rs[step]);

        let q_times_s = multiply_poly(&new_q, &ss[step]);
        let mut new_s = ss[step - 1].clone();
        for _ in 0..(q_times_s.len() - new_s.len()) {
            new_s.push(Rational::from(0));
        }
        for (elem_s, elem_q) in new_s.iter_mut().zip(q_times_s) {
            *elem_s -= elem_q;
        }

        let q_times_t = multiply_poly(&new_q, &ts[step]);
        let mut new_t = ts[step - 1].clone();
        for _ in 0..(q_times_t.len() - new_t.len()) {
            new_t.push(Rational::from(0));
        }
        for (elem_t, elem_q) in new_t.iter_mut().zip(q_times_t) {
            *elem_t -= elem_q;
        }

        if debug {
            println!("r:{:?} \ts:{:?} \tt:{:?}", new_r, new_s, new_t);
        }
        qs.push(new_q);
        rs.push(new_r);
        ss.push(new_s);
        ts.push(new_t);

        step += 1;
    }

    if debug {
        println!();
    }
    rs.pop();
    ss.pop();
    ts.pop();
    return (rs.pop().unwrap(), ss.pop().unwrap(), ts.pop().unwrap());
}

pub fn fast_power(x: &Integer, e: &Integer, n: &Integer) -> Integer {
    let mut res = Integer::from(1);

    let mut base = Integer::from(x % n);
    let mut e = Integer::from(e);

    while e > 0 {
        if e.get_bit(0) {
            res *= &base;
            res %= n;
        }
        e >>= 1;
        base.square_mut();
        base %= n;
    }

    return res;
}

pub fn crt<'a>(
    vals: impl Iterator<Item = &'a Integer>,
    mods: impl Iterator<Item = &'a Integer>,
) -> Integer {
    let mut vals = vals.peekable();
    let mut mods = mods.peekable();

    let mut n = Integer::from(mods.next().unwrap());
    let mut a = Integer::from(vals.next().unwrap());

    while vals.peek().is_some() {
        let n2 = mods.next().unwrap();
        let a2 = vals.next().unwrap();

        let (m1, m2) = bezout(&n, n2);

        a = &a * m2 * n2 + a2 * m1 * &n;
        n *= n2;

        a %= &n;
        if a < 0 {
            a += &n;
        }
    }

    return a;
}

pub fn coppersmith(f: &Vec<Integer>, n: &Integer, m: u32, epsilon_denom: u32) -> Integer {
    let debug = false;

    let d = f.len() as u32 - 1;
    // let m = n.significant_bits() / d;
    let w = (d * (m + 1)) as usize;

    let x_exponent = epsilon_denom - d;
    let x_root = d * epsilon_denom;
    let mut x_powers = Vec::new();
    x_powers.push(Integer::from(1));
    x_powers.push(n.clone().pow(x_exponent).root(x_root));
    for _ in 2..(d * (m + 1)) {
        x_powers.push(Integer::from(x_powers.last().unwrap() * &x_powers[1]));
    }

    // if debug {
    println!(
        "d = {}  N = {}  1/e = {}  X = {} ",
        d, n, epsilon_denom, x_powers[1]
    );
    println!("m = {}  w = {}", m, w);
    println!("{}", x_powers.len());
    // }

    let mut basis = Vec::new();
    let mut det = Integer::from(1);

    if debug {
        println!("starting with basis");
    }
    for v in 0..(m + 1) {
        for u in 0..d {
            let mut g_uv = exp_poly(f, &Integer::from(v));

            for _ in 0..u {
                g_uv.insert(0, Integer::from(0));
            }
            if debug {
                print!("  g_{},{} (len {}) [", u, v, g_uv.len());
            }
            for (i, coef) in g_uv.iter_mut().enumerate() {
                if debug {
                    if *coef == 0 {
                        print!("0, ")
                    } else if *coef == 1 {
                        print!("m{}x{}, ", (m - v), i);
                    } else {
                        print!("{}m{}x{}, ", coef, (m - v), i);
                    }
                }
                *coef *= Integer::from(n.pow(m - v));
                *coef *= &x_powers[i];
            }
            det *= &g_uv[(u + v * d) as usize];
            for _ in g_uv.len()..w {
                g_uv.push(Integer::from(0));
                if debug {
                    print!("0, ");
                }
            }
            if debug {
                println!("]");
            }
            basis.push(g_uv);
        }
    }
    if debug {
        println!();
    }
    let exp_two_w4 = Float::with_val(1024, 2).pow(w as f64 / 4.0);
    let left_bound = det.root(w as u32) * exp_two_w4;

    let right_bound = Integer::from(n.pow(m)) / Float::with_val(1024, w).sqrt();
    println!(
        "condition:{} frac {:.3}",
        left_bound < right_bound,
        left_bound / right_bound
    );

    let (reduced_basis, _min_idx) = lll(&basis);

    if debug {
        for v in 0..(m + 1) {
            for u in 0..d {
                print!(" v_{},{}  [", u, v);
                for coef in &reduced_basis[(u + d * v) as usize] {
                    print!("{:}, ", coef);
                }
                let norm = inner_product(
                    &reduced_basis[(u + d * v) as usize],
                    &reduced_basis[(u + d * v) as usize],
                )
                .to_f64();
                println!("] norm {:.2e}", norm);
            }
        }
    }

    for reduced_poly in &reduced_basis[0..2] {
        let search_range = 10;
        let guesses = approximate_zero(reduced_poly, &x_powers[1]);
        for guess_x in guesses {
            // if debug {
            //     println!("guess x={}", guess_x);
            // }

            for i in -search_range..=search_range {
                let x = Integer::from(&guess_x + i);
                let f_of_x = eval_poly(&x, f, n);
                if f_of_x == 0 {
                    return x;
                }
            }
        }
    }
    return Integer::from(-1);
}

pub fn lll(basis_integer: &Vec<Vec<Integer>>) -> (Vec<Vec<Integer>>, usize) {
    let debug = false;

    let n = basis_integer[0].len() - 1;
    let delta = Rational::from((3, 4));

    let mut basis: Vec<Vec<Rational>> = basis_integer
        .iter()
        .map(|vec| vec.iter().map(|x| Rational::from(x)).collect())
        .collect();

    let min_norm = basis.iter().map(|v| l2_norm_squared(v)).min().unwrap();

    if debug {
        println!("rational basis:");
        print_basis(&basis, 0);
    }

    let (mut b_star, mut mu_matrix) = gsp(&basis);

    if debug {
        println!("b*:");
        print_basis(&b_star, 0);
        println!("mu:");
        print_basis(&mu_matrix, 0);
        println!();
    }

    let mut k = 1;
    // println!("{:.^1$}", "goal", n); // performance meter for large computations
    while k <= n {
        // print!("\x1b[2K\r");
        // print!("{:.<1$}", "", k);
        io::stdout().flush().unwrap();
        for j in (0..k).rev() {
            if debug {
                println!("trying k={} j={} mu={}", k, j, mu_matrix[k][j]);
            }
            if mu_matrix[k][j] >= 0.5 || mu_matrix[k][j] <= -0.5 {
                if debug {
                    print!(" mu*b_j = [");
                }
                for i in 0..basis[0].len() {
                    if debug {
                        print!("{:?}, ", Rational::from(&mu_matrix[k][j]).round());
                    }
                    let to_subtract = Rational::from(&mu_matrix[k][j]).round() * &basis[j][i];
                    basis[k][i] -= &to_subtract;
                }
                if debug {
                    println!("]");
                }
                gsp_efficient(&basis, &mut b_star, &mut mu_matrix, k);

                if debug {
                    println!(" rational basis:");
                    print_basis(&basis, 1);
                    println!(" b*:");
                    print_basis(&b_star, 1);
                    println!("mu:");
                    print_basis(&mu_matrix, 1);
                    println!();
                }
            }
        }

        if inner_product(&b_star[k], &b_star[k])
            >= (&delta - Rational::from(mu_matrix[k][k - 1].square_ref()))
                * inner_product(&b_star[k - 1], &b_star[k - 1])
        {
            k += 1;
            if debug {
                println!("increment k to {}\n", k);
            }
        } else {
            basis.swap(k - 1, k);
            gsp_efficient(&basis, &mut b_star, &mut mu_matrix, k - 1);

            k = std::cmp::max(k - 1, 1);

            if debug {
                println!("swap {} {}", k, k - 1);
                println!("rational basis:");
                print_basis(&basis, 1);
                println!("b*:");
                print_basis(&b_star, 1);
                println!("mu:");
                print_basis(&mu_matrix, 1);
                println!();
                println!("k to {}\n", k);
            }
        }
    }
    println!();

    // check ∀1≤i≤n, j<i. |μ_i,j|≤1/2
    for i in 0..basis.len() {
        for j in 0..i {
            if mu_matrix[i][j] > (1, 2) {
                println!("mu_{},{} was {:.3}", i, j, mu_matrix[i][j].to_f32());
            }
            assert!(mu_matrix[i][j] <= (1, 2));
        }
    }

    // check ∀1≤i<n. δ‖ ̃b_i‖2 ≤ ‖μ_i+1,i  ̃b_i +  ̃b_i+1‖^2
    for i in 0..(basis.len() - 1) {
        let lhs = &delta * l2_norm_squared(&b_star[i]);
        let mu_bi: Vec<Rational> = b_star[i]
            .iter()
            .map(|elem| Rational::from(&mu_matrix[i + 1][i] * elem))
            .collect();
        let mu_bi_bip1: Vec<Rational> = mu_bi
            .iter()
            .zip(&b_star[i + 1])
            .map(|(lhs, rhs)| Rational::from(lhs + rhs))
            .collect();
        let rhs = l2_norm_squared(&mu_bi_bip1);

        if lhs > rhs {
            println!("failed on b_{0} > ub_{0} + b_{0}+1", i)
        }
        assert!(lhs <= rhs);
    }

    let basis_output: Vec<Vec<Integer>> = basis
        .iter()
        .map(|v| {
            v.iter()
                .map(|elem| Rational::from(elem).round().into_numer_denom().0)
                .collect()
        })
        .collect();

    let mut v_norms: Vec<Rational> = Vec::new();
    if debug {
        println!("min norm^2 is      {}", min_norm);
    }
    for (i, v) in basis_output.iter().enumerate() {
        let v_rational = v.iter().map(|elem| Rational::from(elem)).collect();
        let v_norm = l2_norm_squared(&v_rational);
        if debug {
            println!("v{:2} norm^2 is      {}", i, v_norm);
        }
        v_norms.push(v_norm);
    }
    let min_norm = v_norms.iter().min().unwrap();
    let min_idx = v_norms.iter().position(|x| x == min_norm).unwrap();
    if debug {
        println!("min reduced norm^2 {} (idx {})", min_norm, min_idx);
        println!();
    }
    return (basis_output, min_idx);
}

fn compute_mus(basis: &Vec<Vec<Rational>>, b_star: &Vec<Vec<Rational>>) -> Vec<Vec<Rational>> {
    return basis
        .iter()
        .map(|v| {
            b_star
                .iter()
                .map(|v_star| {
                    Rational::from(inner_product(v, v_star) / inner_product(v_star, v_star))
                })
                .collect()
        })
        .collect();
}

pub fn gsp(basis: &Vec<Vec<Rational>>) -> (Vec<Vec<Rational>>, Vec<Vec<Rational>>) {
    let debug = false;

    let mut max_denom = Integer::from(0);

    let mut new_basis = Vec::new();
    let mut mus = Vec::new();
    for (i, vector) in basis.iter().enumerate() {
        let mut mu_row = Vec::new();
        if debug {
            println!("reducing v{} ({:?})", i, vector);
        }
        let mut u_n = vector.clone();
        for j in 0..i {
            let (sub, mu) = proj(&new_basis[j], vector);
            if debug {
                println!(
                    " subtracting proj_{}{:?} ({:?}) = {:?}",
                    j, new_basis[j], vector, sub
                );
            }
            for (k, u_val) in u_n.iter_mut().enumerate() {
                if *sub[k].denom() > max_denom {
                    max_denom = Integer::from(sub[k].denom())
                }
                *u_val -= &sub[k];
                // limit_precision_mut(u_val, 64);
            }
            mu_row.push(mu);
        }
        if debug {
            println!("reduced v{} to {:?}", i, u_n);
        }
        new_basis.push(u_n);
        mus.push(mu_row);
    }
    if debug {
        println!("max denom {}", max_denom);
    }
    return (new_basis, mus);
}

pub fn gsp_efficient(
    basis: &Vec<Vec<Rational>>,
    b_star: &mut Vec<Vec<Rational>>,
    mus: &mut Vec<Vec<Rational>>,
    mut updated_row: usize,
) {
    let debug = false;

    while updated_row < basis.len() {
        let vector = &basis[updated_row];
        let mu_row = &mut mus[updated_row];
        mu_row.clear();

        if debug {
            println!("reducing v{} ({:?})", updated_row, vector);
        }

        let mut u_n = vector.clone();
        for j in 0..updated_row {
            let (sub, mu) = proj(&b_star[j], vector);
            if debug {
                println!(
                    " subtracting proj_{}{:?} ({:?}) = {:?}",
                    j, basis[j], vector, sub
                );
            }
            for (k, u_val) in u_n.iter_mut().enumerate() {
                *u_val -= &sub[k];
                // limit_precision_mut(u_val, 1024);
            }
            mu_row.push(mu);
        }
        if debug {
            println!("reduced v{} to {:?}", updated_row, u_n);
        }
        b_star[updated_row] = u_n;
        updated_row += 1;
    }
}

fn proj(u: &Vec<Rational>, v: &Vec<Rational>) -> (Vec<Rational>, Rational) {
    let mut ret = u.clone();
    let uv = inner_product(u, v);
    let uu = inner_product(u, u);

    let mu = uv / uu;
    for val in ret.iter_mut() {
        *val *= &mu;
    }

    return (ret, mu);
}

fn inner_product<'a, T, U>(u: &'a [T], v: &'a [U]) -> Rational
where
    &'a T: std::ops::Mul<&'a U>,
    <&'a T as std::ops::Mul<&'a U>>::Output: Into<Rational>,
{
    return u.iter().zip(v).map(|(un, vn)| (un * vn).into()).sum();
}

fn print_basis(basis: &Vec<Vec<Rational>>, indent: i32) {
    for vec in basis {
        for _ in 0..indent {
            print!(" ");
        }
        println!("{:?}", vec);
    }
}

fn l2_norm_squared(v: &Vec<Rational>) -> Rational {
    return inner_product(v, v);
}

fn derivative(f: &Vec<Integer>, const_x: &Integer) -> Vec<Integer> {
    let mut to_ret = Vec::new();
    for i in 1..f.len() {
        let val = Integer::from(i as u32 * &f[i]) / const_x;
        // println!("{}", val);
        to_ret.push(val);
    }
    return to_ret;
}

fn approximate_zero(f: &Vec<Integer>, const_x: &Integer) -> Vec<Integer> {
    let debug = false;

    let mut results = Vec::new();
    let f_prime = derivative(f, const_x);

    if debug {
        println!("f' = {:?}", f_prime);
    }

    let n_guesses = 50;

    let xs: Vec<Rational> = (0..n_guesses)
        .map(|i| Rational::from(i * const_x) / n_guesses + 97)
        .collect();

    if debug {
        println!("\n{:?}", xs);
    }

    for mut x in xs {
        let mut f_of_x = eval_rational_lattice_poly(&x, f, const_x);

        if debug {
            println!(
                " starting x to {:.3} \tf(x) = {:.1e}",
                x.to_f64(),
                f_of_x.to_f64()
            );
        }
        let mut count = 0;
        loop {
            let denom = eval_rational_lattice_poly(&x, &f_prime, const_x);
            if denom == 0 {
                println!("breaking");
                return results;
            }
            let to_sub = f_of_x / denom;
            x -= &to_sub;
            x = limit_precision(x, 4096);

            if to_sub <= (1, 256) {
                count += 1;
                if debug {
                    println!("newton finished in {} steps", count);
                    println!();
                }
                let int_x = x.round().into_numer_denom().0;
                if !results.contains(&int_x) {
                    results.push(int_x);
                }
                break;
            }
            f_of_x = eval_rational_lattice_poly(&x, f, const_x);
            count += 1;
            if debug {
                println!(
                    " trying x to {:.3} \tf(x) = {:.1e}",
                    x.to_f64(),
                    f_of_x.to_f64()
                );
            }
        }
    }

    return results;
}

fn limit_precision(mut x: Rational, shift: i32) -> Rational {
    x <<= shift;
    x.round_mut();
    x >>= shift;
    return x;
}

fn limit_precision_mut(x: &mut Rational, shift: i32) {
    *x <<= shift;
    x.round_mut();
    *x >>= shift;
}
