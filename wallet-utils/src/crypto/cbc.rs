use aes::cipher::BlockEncryptMut as _;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use base64::prelude::*;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

pub const AES_KEY: &str = "u3es1w0suq515aiw";
pub const AES_IV: &str = "0000000000000000";

pub struct AesCbcCryptor {
    pub key: Vec<u8>,
    pub iv: [u8; 16],
}

impl AesCbcCryptor {
    pub fn new(key: &str, iv: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
            iv: iv.as_bytes().try_into().expect("IV必须是16字节"),
        }
    }

    fn create_cipher<T: KeyIvInit>(&self) -> Result<T, crate::Error> {
        T::new_from_slices(&self.key, &self.iv).map_err(|e| crate::Error::Crypto(e.into()))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<serde_json::Value, crate::Error> {
        let encrypted_data = BASE64_STANDARD
            .decode(encrypted_data)
            .map_err(|e| crate::Error::Crypto(e.into()))?;

        let decryptor: Aes128CbcDec = self.create_cipher()?;
        let buf_size = encrypted_data.len();
        let mut buf = vec![0u8; buf_size];
        let decrypted_data = decryptor
            .decrypt_padded_b2b_mut::<Pkcs7>(&encrypted_data, &mut buf)
            .map_err(|e| crate::Error::Crypto(e.into()))?;

        let decrypted_str = String::from_utf8_lossy(decrypted_data);
        crate::serde_func::serde_from_str(&decrypted_str)
    }

    pub fn encrypt(&self, raw_data: &str) -> Result<String, crate::Error> {
        let data = raw_data.as_bytes();
        let encryptor: Aes128CbcEnc = self.create_cipher()?;

        let buf_size = data.len() + 16;
        let mut buf = vec![0u8; buf_size];
        let encrypted_data = encryptor
            .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buf)
            .map_err(|e| crate::Error::Crypto(e.into()))?;

        Ok(BASE64_STANDARD.encode(encrypted_data))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aes() {
        let iv = "0000000000000000";
        let key = "u3es1w0suq515aiw";

        let encrypted = "YXUhkjy6ZTM9HlZwF8OZDFaisYXZr/3/hDRXU1TwCB5VZqegQoJOeevEMRodVPo2xxMR6ePsBwNeYgl/CQK2j7dvMw8aXwGvDUbEjd8looF+OG1/2N42mvODbR7odPPZfBZWlPT0JuVSTa4My3T+MtEdyQnS5UaJNC+s1T0Haq2DI61WjtH8HuXrZdP8djfJLIKmSQ6QTxrpxvQ06EyAFUzJXSylGj4aaPc/JGFy9WdMXMtChaiaFTwAg0VM9Hvv5xE2B8ONYF3Z1p/ZUsqLQTkXrwcqX01IBckDNRG3dNRY4vtPWyydcxxpWLMWiPAZvUVHj9Ljd6IH5w9XsuIC8BIn8EJsHRFMOBY5nUwivKuoxEfPIWvkT0r7s/ZT/PVypfSeHgEWynrNsFEbMV4BKjcBkPm9nGNBXSoPnkC2kMkcnZfEGEu7Z+rT6gsv/Q6Nxh7aCXHF3qi9+hiYhfMcwhoSdhkWaiQEXOPtR1/m5QxayWs/PWjX138+OInr+tRfNG2K5orkh8NwfVdTKiSIWQKW7SOD+XKfNQIC8O5AenG/gE3C25TZ9LjCQQISG22KObLoLR3N2joA2PyB268JjnfTnZV9U0TJmAuD5yA4QfvDNpbOdBkRyBSjo/1mGhZGy49t6rD6oD3DYmhBM6Kipq0D0GVHyg19sOpCCfnE68F1zQbd0ISMcQ9wW99c0UQn6SW7aVfHLNVJzBihulVpUoA/KR5582gTdmNenixV2inmxdC3S/tljd/Dgf0m5c0qb2wKgdzPTIETIbgoB5nLit3cz2HnWr7rjUJyoxeXRFs251+GHeeXQjATPrcKJaKlIwtS5Sdl76HJssORyvcUyyU5ZLs0xadoSakKlfWNcrHk8EwUw8ZW8DLOqTFFEwYgnMY+O/+BqJCF7HCNijTmBeLK7GzWrqIn04IWlhxYBh1CIdnpr3WinZa9q0wDFbAnMTUVTPW5FsAAuWctzg+c1yV5FXHaC5u+fv7OKCxXz/uEBMDmoVL3wu//2Pv8dgrttbanv11ulWHusFSbRcMSXaLqyvwz4jgarvKLfpZ4FrHZ/cxK6aAjQ2RxcalWWbJutCnaA7UIuA5KeesFNz8JUpk5PBJoC803BfPZPvQN7jZ5wPe71Vxkrj+BN/jYI+ZF4PHqNrIjLxnOoAz1KX07WS6hS09Ivz6n/rEM0/v9f/hgarLv+ysO/DVngCyJzz9AhLTiI+9CpIyU+DSnujbVtRfs+39BKsib5miG4xvlSjaogWxTOZT67FtzX5ABxy3Vr7fb7+NXE6SiI26qH+voV0r4aM8eSQmYH8jNflvmc7L7FMwEI4TRYR6sVgwZPQ0nJXdIapVWVmbN773i6hZgJ+ql8ANqJAX2h01+XnT6y30amlMDZLXUZcnLSku022Vmk2h/3XIQESJ0Omz0nr1+73EV8+c002qBGTRdrnkVj4G/2uIoD/u/Yxb5Uptk/96fz2qThuTGWfcdoOHkr5Mqrd1vGJN7uY0+kEQGHrLOUl0lhoGel/sluwRGyGLglWBFb6GVJvCFHmANPhMuX1T7mowIEwvwEllfF3x/0pUI0YL1rgWFSE+7oPC3I+uP7JcmQYk5Hs97wsxachYHPnE/pQmHGty6JYAixsIUIMQ9ZKlXj6VpGJsLH3IdXj+K3wzP47sxwC6nJY+yiojmJsjYMipXUTBePb2uC5eQL7AP8yhg+O/mjaFWC3zMfD+kjeKSIGHbfIuAWnXBEbiMHObUNnLNPdEKQuzEVUnU3t5WtuOLp1+YMcLRWCfkhmXtCIxtXTj4+VarFRacyYjel2Uzi5mcfQNFv8PdqxIvqIzl4dxNz/27pwyNCFm6tlIECgwZJx5JPOIKhlg00dCsyx7j2sCHWIbF4WLADbT0H/Jiu+LOU6vvFbSTjtul5iMJxiYvhsqbVX5gkGMerD0ZL9wOgp2BFsl+q5a6nmImXTFgxstP+ISuwuPF1TTlxw25KbphzjcgDAZiI3BlkIOBtTLXs4G+6Hmzx5zN/zCxzaM1akt7KJ3qqhm2LKWpYRNcB4K1/hRQ1H2ksd2fRsoR2A5pAais55YuyZuwdV3p+HJrFJ7W9t1b4+K2nOGG/3LA7d7eqebBItKGnRtJNHDW4kagLYG3z+com5W/5pgAXa/a2WV11XhrV/LuiUhlDpyDNRYvr60OvSxo4Irh5/dswaMDauTlSlNhAeUySm/67QJRG+/sapEw1JhSvGHUqY+xoVmv457k+Xf0wVMCf3ohkxbFGsAb9Vwk6HgtW7koUzKoSxht1SDWYMWpMfjgONc4pP2ScAnPMFqs0mqWx3IRKgf7hI9OtH4AixnIAfN3wtC+9hsE8K4HkFnRO2uIwPs9EiqHaC0YChskROhqvYGA1f4Yvvnp+1BDIIF919QJEda9kIh5908OtR+tvPo3ITaN61Exqvf4ROMeITBBSb2L9Cqh8dlRTLRek7p63yo9+VofLCNnNEmf3NaNbI2Uk4hFCfZ5qDIB/yOwdj9gey4JooVtlWJTuYy8ieED9eDQyH0PG7+n4NxmTEYKwCLH2MXqI5BbPjeumHKzSvCxf97AOzIH0eKA5M2zy1IKeuXavDfd8u4YSXvAJBVaZmZyk7aAFers/4a0zASJVH/axxha8qtAd/A9Go193ivGkQkPvnJrbM70nQOYYtpUP2DGij+ti6goRFNFoeQkTx4r3nh5cF1ETx8RJrxEiVIgWge4xzqlEJuP+55uwpOXh9rijFNopC2qy6hk5prVblkpCvC0uSDSin4RGDZVGz48l50uCRXwkMz7qHGrsrcE6LL1HPrrorPjvMGFHu4MBoQA30IEPXmkpdTs7xLpsE63UOgx47tiyvmOBenq1m9rzgo/ERNpkWEJd2CwR5GITy8w1yEttSiO1ahC2wFW9qYJ5NRLd9+XJR/HLWbL0HJtEaHHK3kYYppnPumLB2MXsBe5so3jmj69Y95Ws3HcVGQczFyNkxDwBoXH5gyTFCrGKubEV3ROFvtVnbDT122HEhTnVrtlM4Sn7nTJydFZjKe4Ou27hkjI5lmrBld/yUMgUSa4W1JuhNmHZMoUekAnT5V5fftVQmhbdlKKKs/ZDtsnEwtV1zmQn872ytlnhN6jJFspwMxxWaVLVesbk1DoM5vTHLOzU2azHKEJHcagnRRfj9GYe/fn/nOpE9wLya9d9tzNc9oUbLdbzZ3kcYYF1dEsvJRjeuypjSSGeEemVtQQW1IfbHIykWnwNPQVCnYsVyAsUDDxjkt0HIjggfA7oxpz/nUXfljyc0t4CencKJdS2C5ntDzYHmu0QlOGBQI/b2NI91wcTPmrWtRoYjRqbhb31ceLUyw1qf71oalqWA/sWsB91HXykZ3I/pyfSuRW01AZhRp/HWj7QGPLrHK7jk1Gmq8YC9gUKYPs0EutE2gQle3hzU3GYx6YGCP11RAkeHKP8py+EHiQzAOGVg/6T4PNqPyRCvHzqP93ox8DyuuGizLBv7Z4duStu6mj9btI67KqqpEMUALU6z8HQqslDbwye9Q6ui/X03aNz1PXR74QPH+9pQStpM/sgbE15FZMQES/q9MMrPOjIUTyvkUaLnV3ctRRlDBWVuZPLjvFHlWFdHp7Fs89JX9tCehNf5MOiErwKOTNom40oBilwcwmICavu/KttYdEZZutFHFe2znroEBcRZbJwCTerTqJLnMvVxXLJnMRbCu9UE6IldGXCizNRqAwAYqwNdjCpjGzlpH3XFYWNSPHnYEQCFa6Wc130gF0UzKFVpzPYjmCxDdJ6pbRVSIg3EWvfrSVl8F/oTTXuckMnX4tav47sOStIsRHLe6994KlEz4mn1HIdpw6wTqZgq1WBgnKrVe7rjLYq83NjG6roPNCvZGYYP08exTKiPg3+fHDyST+xzdSkSXLkRGv01q3EjpRds9pBDmvw1nSonUn206saI18UtOC3TmhC1rgNRrf/+sEPmeKlSsxYlkpqjoyjRzRYbwD/V3HgAY4xDx9lhupX2Iy4MjdEUQH0ggSZwpk90a52zVrmZFGDsfEw2a5VMVVEs89mq4xJaO1Ajlx1WEkMCbw52zG4MG3t3FhWsC7vhcVjGMHsYRN//t+LMS4HEdByxp5asanyZJ5tyMn0AjyFkiiI/jgs1kMzCJVQHS8tXZMTCjHTybLOC6cqiJSKqT22RwFBV1r4x1cCZixCkeCIgUhVWC3hrTVVtcJvq/5mu1oDTM1pH/0cSTvep6CR2xhYdL6xgmKntAJgvWlABiaH2rTHny0tI2HUXAodHOoPDRcWRnQFAykltYkfdZ0WXpqdGSRKKngu/RfgkkPl7jZyZDuvg+vfYaB12DhiHSpCNuPbubso8CuzElQyJ/DJbFWbY3aTtXilH8ez0hucoHCc1h4WSCOoeiPmxg99eblyXLN6zG82gIG0WE1mUmzB7PspY2dcE9/+n0FcaR5IBRTLtY6kC2wbmJ/QgK81goJoN4nk0RA6rTYXq5lmIuFVs1gmwiLfhWStrAQB9ig5hnq3XsscFEETB+gG4d91nvq+wfGMtyYliGQJZEdHeCN/BZlXrrU+BfsAkRxQsYR6Oxs7UE/PVQ6Vj71ZsMGirpIaLiKDvOgy0BWd2aZWLH5dQT1pTRvyHYHPW67fENU2jsiHOftTy3BWJhqJOZOy5wmgemRkCxQ+22IZJbdT4Y8K0GwGxJin3rfvkUl+5mc/SDjSvgwS3f5qiq31MJY4xL6f4wdE/vK+K5VCFEi5yU39f/kP9Kt7qzqZtzBId0TVVHHcOMirYUBpIhiiL+QnklPE/yUcMwJ/5fb99TNZrHYZoGyCV0KDMWbXiafH8AXm2x7CX/RxIAUcfDPnf7mnWHmYs0OWGEveRpbmoiVBcxGevhG1KFnDr5mbsMrFMjb1vYCwJ9XP1lHKDlkybM/hdn4nVz98xXl6I6RuXTK7IXrvxyNkVssZ3lFrx1UMOKxDLf8v/kIzecpJeazTP1ej+HJLUmyGCRvWfOAPP0iu4GCNps9Kd3q9+lZxbGLJ4qngAaqmjpTI4Cy/IURoasDeRQF69uYLjR/R3iHcpbbPe4TEiMBPuokStWZG3I8Ffs4F0GPS04HJmv8XtwoyGLED9Gubr52nFqQ3dXvcxHsVcCb/CB3PV3hHM09fGXU2xLzH2HJ7NhMG3AzUq802iL9r6znZ/d9uQ55yLnw69ackNcrqNzJChPSe69d10kY8q/Cc6QMqOzDutS/NN/Qq21cK8mu3PLxaE0GwM+KMpsMy+Qzzg7zsmNeaqdjqv27n7/y7hqThM7UeQ6WDQPDBCBoyv24bMFOXl2Tv29n+/RR5ztN0p9hLSJhGY26ZIBS5v2nuPA047MjlnSY8k14Q/oI3Tu6ipTZOvz3NfJC9hQoM/PSV4KCdQKT3dq5+UwRx6hitX2690O6R2nZRQQGrOUQgrKuIh6UnPCPo026Gfl6DqmxeTIFTCNTIswZZF5++gvCFmganRO3dDzoflmZOGDnjSEHKwNP+bVG948RnK1pPIeRxDNy2PNEjE0Za5XmiWLNQfhqEbC4FSjgfE+W7bWl8e8wo52ICoEWVM17NaXXXSjxA4OTlXf+V0iuDOJqHSc2FF1kH1fp6LqjUEieTRynY0U8tQ2NUzDpGjts2aO4gAnvNSFBg6ST/faJOmrRbSauhTch51VEjG3odHHoZZKjOSjDJ7JYdYd7oM1jrmy6POLoPdDzBQK+wpBC6p0Epfdtv+uUveUNLD7fz8dmhfuqMjAqBwiZ76+cFrlqMKs6lpPdLCbQ3bc1P9/Cd6smQey2FvEo2S9M0VJgs2frJS9AwNBA6olktXo8wAQp7kQaqY+OPMi2+2Q2D0EQfviAXIz7VimPx8KudrXK9dmiJsoCaLMuLEJoeXMPr1kWOE6u9B4H7XdIsUeBgJNpxbwo0XY25IF/H8nuRjiJZh/yndDgwALWy1IqguVw+PCxrZ6gx4OV6l1EN1wrSWenmyz6eU3laQ6OaaDUg72my808iokr2WGaTKwVcOt957dkWEC4fVUnYfmVO3Ka965jZyLxMJRNdsWBcauPRZ5TouSg6RtDs3s9dEIgSOYo3VLaabaSwW7TTst2r3n56zp19sYULIOoOSu+Q9w//bwty2q2iVc3tqmYVvFBbmLj5aaQ14QFdKvZATApCfXniO3WhIplFmD+HOJLPVTDGZh35Zs81fUAGIRsn7EiQQ/rpuHnGzisxKKPCObHrvxBUIip7LTJKDCjFqBzIXsq55Fp2ShDqyB/hPlSCJ5aEhvp3acpq833wCOlPxZGiylOLbRxQMF15tagXS3ggO4X2PFGh5gYp64beCQ1aEDDOaBhen/hKsiEM//UcbOCm4gJdTdQ3okK8tATW68+Zp5fNDcY1iu2dd/2HQjT9DKMcUgeQpPOnTADrevZXDVgrDe14iuf8Q1nHZifMWnwg/3S+6blz5zLHwfWMSv37iDsyw1XE5Pa9xlOrpPqj/zYrsdAEckuG/TfYyZ0nivnvjByMyphCiYlJzz82rHtIPsF4lWF36IPgqflEPEnJ6B7A+S4jUGBK+oyOon7tM8M8edj/F39CXv6N8NxoOkExQ5UJbExpAmCLm+ttX40jxJUUsjEDCYjkwZC4D9GRAXLud+brRvTImFmcucNwUYXpcbLrvl2ujAKkmKpClMtwvyZrELQvHElLHQti/RTKwulJjTlvLTYk31w+cBLDIGiFA4xxVXgHrNeJypbeusLwkvsFT7/Wcy2Wu5Ji4ZBMdxszpyuF53LUiZNE4L5X/MtPbYWlwPR0uKODl49OZf16I0q7TL0c+yZ6rv9bf/y7Fuft+0v3V/y5vtMnnb2r0rScz30TKCDJYmq2YF294Waw72qknCesOC6gymK20uyXdlMS+W8uhc/+axTrnJQUCyrrk0eyXA5Jdhv2NSOqCYz5eC4KJQzeCtYbc87lUhYkbx6jwiY9yD747NrLNw/bzjcLqkvdyaZnu9xwnD0osQy5MCJ0Wa9aULd3upjYaOC+XzXVNsy2Mkin2CzkLwY314p/O0f+Vw1MhVMpexWO0iwsKZEuNVuzgfDHsbYgemJhyvJqBiTiZ0Gn8NNkyCbS7OEV7TsH4m7W11rJVX7O9lfwPgEA7nLn/HcPNd87xxLydVQWdYGYr7AQoNSMqfzot+/uThVhxJjlcr/wntAJiI/E3Z//5mYiuYPHGN2EGjoikRo9gi2ESErk9jymHVxm3SYJme/7GmEakald+6UEDphSIJj450uK2eh8izLwqK7VWh54FmCR33CvFMLVkkXLe7H8Yd6IXEFz8EN6vgFvb6RXc/PWINIcQg0V7GkNNH/WvHZMVO98DzgMw8+6BaHvTOrIDJgqEgIYDR/6q8U4Wzu3BRS53jt3aMDp/IFxaeHFScrXNkovI573DmYFEzKVK8J4JHh+hQDWJK5Gf/IhkIWU9LHZ4KIJsZZynBoZYdpxLXzocE+8GgjplDp6rbyAu5N9YmuNOjGOjR0oyRo4ququ8/WlsY7ccJHMgYKYNGazn116RQveyv+/MaTxTgTmMHdjVRz0sjRexHywpMKAyLPiUgJkg==";

        let decrypter = AesCbcCryptor::new(key, iv);
        let _res = decrypter.decrypt(encrypted).unwrap();
        // println!("Decrypted: {}", res);

        // let res = decrypter.encrypt(&res).unwrap();
        println!("encrypted: {:#?}", _res);

        // assert_eq!(encrypted, res);
    }
}
