#[derive(Debug, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, PartialEq)]
pub struct Cmyk {
    pub c: f64,
    pub y: f64,
    pub m: f64,
    pub k: f64,
}

fn max(rp: f64, gp: f64, bp: f64) -> f64 {
    let rg_max = if rp > gp { rp } else { gp };
    if rg_max > bp {
        rg_max
    } else {
        bp
    }
}

pub fn convert_rgb_to_cmyk(rgb: &Rgb) -> Cmyk {
    let rp = rgb.r as f64 /255.;
    let gp = rgb.g as f64 /255.;
    let bp = rgb.b as f64 /255.;
    
    let k = 1. - max(rp, gp, bp);
    if k == 1. {
        return Cmyk {
            c: 0.,
            y: 0.,
            m: 0.,
            k: 1.,
        }
    }
    Cmyk {
        c: (1. - rp - k) / (1. - k),
        y: (1. - bp - k) / (1. - k),
        m: (1. - gp - k) / (1. - k),
        k,
    }
}

pub fn convert_cmyk_to_rgb(cmyk: &Cmyk) -> Rgb {
    Rgb {
        r: (255. * ((1. - cmyk.c) * (1. - cmyk.k))) as u8,
        g: (255. * ((1. - cmyk.m) * (1. - cmyk.k))) as u8,
        b: (255. * ((1. - cmyk.y) * (1. - cmyk.k))) as u8,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_cmyk_conversion_white() {
        let rgb = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(0., cmyk.c);
        assert_eq!(0., cmyk.y);
        assert_eq!(0., cmyk.m);
        assert_eq!(0., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_black() {
        let rgb = Rgb {
            r: 0,
            g: 0,
            b: 0,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(0., cmyk.c);
        assert_eq!(0., cmyk.m);
        assert_eq!(0., cmyk.y);
        assert_eq!(1., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_red() {
        let rgb = Rgb {
            r: 255,
            g: 0,
            b: 0,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(0., cmyk.c);
        assert_eq!(1., cmyk.m);
        assert_eq!(1., cmyk.y);
        assert_eq!(0., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_green() {
        let rgb = Rgb {
            r: 0,
            g: 255,
            b: 0,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(1., cmyk.c);
        assert_eq!(0., cmyk.m);
        assert_eq!(1., cmyk.y);
        assert_eq!(0., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_blue() {
        let rgb = Rgb {
            r: 0,
            g: 0,
            b: 255,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(1., cmyk.c);
        assert_eq!(1., cmyk.m);
        assert_eq!(0., cmyk.y);
        assert_eq!(0., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_yellow() {
        let rgb = Rgb {
            r: 255,
            g: 255,
            b: 0,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(0., cmyk.c);
        assert_eq!(0., cmyk.m);
        assert_eq!(1., cmyk.y);
        assert_eq!(0., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_cyan() {
        let rgb = Rgb {
            r: 0,
            g: 255,
            b: 255,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(1., cmyk.c);
        assert_eq!(0., cmyk.m);
        assert_eq!(0., cmyk.y);
        assert_eq!(0., cmyk.k);
    }

    #[test]
    fn rgb_cmyk_conversion_magenta() {
        let rgb = Rgb {
            r: 255,
            g: 0,
            b: 255,
        };

        let cmyk = convert_rgb_to_cmyk(&rgb);

        assert_eq!(0., cmyk.c);
        assert_eq!(1., cmyk.m);
        assert_eq!(0., cmyk.y);
        assert_eq!(0., cmyk.k);
    }
}