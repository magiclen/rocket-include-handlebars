use crate::handlebars::*;
use crate::EntityTag;

#[inline]
pub(crate) fn compute_data_etag<B: AsRef<[u8]> + ?Sized>(data: &B) -> EntityTag<'static> {
    EntityTag::from_data(data)
}

#[allow(unused_variables)]
#[inline]
pub(crate) fn add_helpers(handlebars: &mut Handlebars) {
    #[cfg(feature = "helper_inc")]
    {
        handlebars::handlebars_helper!(inc: |x: i64| x + 1);

        handlebars.register_helper("inc", Box::new(inc));
    }

    #[cfg(feature = "helper_dec")]
    {
        handlebars_helper!(dec: |x: i64| x - 1);

        handlebars.register_helper("dec", Box::new(dec));
    }

    #[cfg(feature = "helper_eq_str")]
    {
        handlebars_helper!(eq_str: |x: str, y: str| x == y);

        handlebars.register_helper("eq_str", Box::new(eq_str));
    }

    #[cfg(feature = "helper_eq_str")]
    {
        handlebars_helper!(ne_str: |x: str, y: str| x != y);

        handlebars.register_helper("ne_str", Box::new(ne_str));
    }
}
