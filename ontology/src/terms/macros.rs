#[macro_export]
macro_rules! matchtm {
    (
        sym(_)
        = $input:expr $( => {$($block:tt)*} $(else {$($else:tt)* })? )?
    ) => {
        $crate::matchtm!(@REC $( { $($block)* } $(else {$($else)*})? )? ; { matches!($input,$crate::terms::Term::Symbol{..}) } )
    };
    (
        sym($name:ident)
        = $input:expr $( => {$($block:tt)*} $(else {$($else:tt)* })? )?
    ) => {
        $crate::matchtm!(@REC $( { $($block)* } $(else {$($else)*})? )?; { $crate::terms::Term::Symbol{uri:$name,..} = $input } )
    };
    (
        sym(=$name:expr)
        = $input:expr $( => {$($block:tt)*} $(else {$($else:tt)* })? )?
    ) => {
        $crate::matchtm!(@REC $( { $($block)* } $(else {$($else)*})? )?; { $crate::terms::Term::Symbol{uri,..} = $input && uri == $name } )
    };
    (
        app( {$($p:tt)*} ,
            [
                $( {$($a:tt)*} $(*$($dummy:literal)?)? $(?$($dummy2:literal)?)? ),*
                $(_ $($dummy3:literal)?)?
                $($args:ident)?
            ]
        )
        = $input:expr $( => {$($block:tt)*} $(else {$($else:tt)* })? )?
    ) => {
        $crate::matchtm!(@ARR app { $( {$($block)*} $(else {$($else)*} )? )? ; }
            { $crate::terms::Term::Application(app) = $input && let head = &app.head }
            [
                $(
                    $(*$($dummy)?)? $(?$($dummy2)?)? {$($a)*}
                ),*
                $(_ $($dummy3)?)?
                $($args)?
            ]
            {
                {head;$($p)*}
            }
        )
    };

    (@REC {$($block:tt)* } $(else {$($else:tt)* })? ; { $($done:tt)* } ) => {
        if let $($done)* { $($block)* } $(else {$($else)*} )?
    };

    (@REC ; { $($done:tt)* } ) => {
        if let $($done)* { true } else { false }
    };

    (@REC $( {$($block:tt)* } $(else {$($else:tt)* })? )? ; { $($done:tt)* }
        {$next:ident;_}
    $($rest:tt)* ) => {
        $crate::matchtm!(@REC $( {$($block)*} $(else {$($else)*})? )? ; { $($done)* } $($rest)*  )
    };

    // sym()
    (@REC $( {$($block:tt)* } $(else {$($else:tt)* })? )? ; { $($done:tt)* }
        {$next:ident;sym(_)}
    $($rest:tt)* ) => {
        $crate::matchtm!(@REC $( {$($block)*} $(else {$($else)*})? )? ; { $($done)* && matches!($next,$crate::terms::Term::Symbol{..}) } $($rest)*  )
    };
    (@REC $( {$($block:tt)* } $(else {$($else:tt)* })? )? ; { $($done:tt)* }
        {$next:ident;sym($name:ident)}
    $($rest:tt)* ) => {
        $crate::matchtm!(@REC $( {$($block)*} $(else {$($else)*})? )? ; { $($done)* && let $crate::terms::Term::Symbol{uri:$name,..} = $next } $($rest)*  )
    };
    (@REC $( {$($block:tt)* } $(else {$($else:tt)* })? )? ; { $($done:tt)* }
        {$next:ident;sym(=$name:expr)}
    $($rest:tt)* ) => {
        $crate::matchtm!(@REC $( {$($block)*} $(else {$($else)*})? )? ; { $($done)* && let $crate::terms::Term::Symbol{uri,..} = $next && *uri == $name } $($rest)*  )
    };

    // arrays
    (@ARR $id:ident { $($pre:tt)* } { $($done:tt)* } [ ] { $($post:tt)* } ) => {
        $crate::matchtm!(@REC $($pre)* { $($done)* &&  $id.arguments.is_empty() }  $($post)* )
    };
    (@ARR $id:ident { $($pre:tt)* } { $($done:tt)* } [ _ ] { $($post:tt)* } ) => {
        $crate::matchtm!(@REC $($pre)* { $($done)* }  $($post)* )
    };
    (@ARR $id:ident { $($pre:tt)* } { $($done:tt)* } [ $args:ident ] { $($post:tt)* } ) => {
        $crate::matchtm!(@REC $($pre)* { $($done)* && let $args = &*$id.arguments }  $($post)* )
    };
    (@ARR $id:ident { $($pre:tt)* } { $($done:tt)* } [
        $(*$($dummy:literal)?)? $(?$($dummy2:literal)?)? {$($p:tt)* }
    ] { $($post:tt)* } ) => {
        $crate::matchtm!(@REC $($pre)* { $($done)* &&
            let [$crate::matchtm!(@MSEQ a $(*$($dummy)?)? $(?$($dummy2)?)? )] = &*$id.arguments
        } {a;$($p)*} $($post)* )
    };

    // arguments
    (@MSEQ $name:ident * ) => {
        $crate::terms::Argument::Sequence($name)
    };
    (@MSEQ $name:ident ? ) => {
        $name
    };
    (@MSEQ $name:ident  ) => {
        $crate::terms::Argument::Simple($name)
    };
}
