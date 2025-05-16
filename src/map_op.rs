//! Helper macro for RangeMapBlaze binary-operator impls.
//!
//! Usage pattern:
//
//  map_op!(
//      bitand &,                            // Rust trait + symbol
//      $crate::map::RangeMapBlaze<T, V>,    // RHS concrete type
//      /// One doc-comment that shows up on every impl.,
//
//      |a,  b|  { /* owned vs owned   */ },
//      |a, &b|  { /* owned vs &borrow */ },
//      |&a, b|  { /* &borrow vs owned */ },
//      |&a,&b|  { /* &borrow vs &bor  */ },
//  );
//
#[macro_export]
macro_rules! map_op {
    (
        $trait_name:ident $symbol:tt,
        $rhs:ty,
        $(#[$meta:meta])*
        $doc:literal,
        |$a1:ident , $b1:ident|  $body1:block,
        |$a2:ident , & $b2:ident| $body2:block,
        |& $a3:ident , $b3:ident| $body3:block,
        |& $a4:ident , & $b4:ident| $body4:block $(,)?
    ) => {
        // ---- owned  ⊛  owned ----
        $(#[$meta])*
        #[doc=$doc]
        impl<T, V> ::core::ops::$trait_name<$rhs>
            for $crate::map::RangeMapBlaze<T, V>
        where
            T: $crate::Integer,
            V: ::core::cmp::Eq + ::core::clone::Clone,
        {
            type Output = $crate::map::RangeMapBlaze<T, V>;
            #[inline]
            fn $symbol(self, rhs: $rhs) -> Self::Output {
                let $a1 = self;
                let $b1 = rhs;
                $body1
            }
        }

        // ---- owned  ⊛  &borrowed ----
        impl<T, V> ::core::ops::$trait_name<&$rhs>
            for $crate::map::RangeMapBlaze<T, V>
        where
            T: $crate::Integer,
            V: ::core::cmp::Eq + ::core::clone::Clone,
        {
            type Output = $crate::map::RangeMapBlaze<T, V>;
            #[inline]
            fn $symbol(self, rhs: &$rhs) -> Self::Output {
                let $a2 = self;
                let $b2 = rhs;
                $body2
            }
        }

        // ---- &borrowed  ⊛  owned ----
        impl<T, V> ::core::ops::$trait_name<$rhs>
            for &$crate::map::RangeMapBlaze<T, V>
        where
            T: $crate::Integer,
            V: ::core::cmp::Eq + ::core::clone::Clone,
        {
            type Output = $crate::map::RangeMapBlaze<T, V>;
            #[inline]
            fn $symbol(self, rhs: $rhs) -> Self::Output {
                let $a3 = self;
                let $b3 = rhs;
                $body3
            }
        }

        // ---- &borrowed  ⊛  &borrowed ----
        impl<T, V> ::core::ops::$trait_name<&$rhs>
            for &$crate::map::RangeMapBlaze<T, V>
        where
            T: $crate::Integer,
            V: ::core::cmp::Eq + ::core::clone::Clone,
        {
            type Output = $crate::map::RangeMapBlaze<T, V>;
            #[inline]
            fn $symbol(self, rhs: &$rhs) -> Self::Output {
                let $a4 = self;
                let $b4 = rhs;
                $body4
            }
        }
    };
}

#[macro_export]
macro_rules! map_unary_op {
    (
        $trait_name:ident $method:ident,       // e.g.  Not  not
        $out:ty,                               // output type
        $(#[$meta:meta])*                      // doc attrs
        $doc:literal,                          // doc string
        |& $self_ident:ident| $body_ref:block  // body for &self
        $(,)?                                  // optional trailing comma
    ) => {
        // ---- impl for &RangeMapBlaze ----
        $(#[$meta])*
        #[doc=$doc]
        impl<T, V> ::core::ops::$trait_name
            for &$crate::map::RangeMapBlaze<T, V>
        where
            T: $crate::Integer,
            V: ::core::cmp::Eq + ::core::clone::Clone,
        {
            type Output = $out;

            #[inline]
            fn $method(self) -> Self::Output {
                let $self_ident = self;
                $body_ref
            }
        }

        // ---- impl for owned RangeMapBlaze (delegates) ----
        impl<T, V> ::core::ops::$trait_name
            for $crate::map::RangeMapBlaze<T, V>
        where
            T: $crate::Integer,
            V: ::core::cmp::Eq + ::core::clone::Clone,
        {
            type Output = $out;

            #[inline]
            fn $method(self) -> Self::Output {
                // forward to the &self implementation
                (&self).$method()
            }
        }
    };
}
