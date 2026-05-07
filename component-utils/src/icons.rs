use leptos::prelude::*;

fn make_icon(
    icon: icondata_core::Icon,
    style: &'static str,
    width: &'static str,
    height: &'static str,
) -> impl IntoView {
    view! {
        <svg
            style=style
            x=icon.x
            y=icon.y
            width=width
            height=height
            viewBox=icon.view_box
            stroke-linecap=icon.stroke_linecap
            stroke-linejoin=icon.stroke_linejoin
            stroke-width=icon.stroke_width
            stroke=icon.stroke
            fill=icon.fill.unwrap_or("currentColor")
            inner_html=icon.data
        />
    }
}

macro_rules! icon {
    ($( $name:ident$(($color:ident))? = $icon:path ),*  $(,)?) => {
        $(
            icon!(@I $name$(($color))? = $icon);
        )*
    };
    (@I $name:ident$(($color:ident))? = $icon:path) => {
        #[component]
        pub fn $name(
            #[prop(default="1em")] width:&'static str,
            #[prop(default="1em")] height:&'static str
            $(,#[prop(default = false)] $color:bool)?
        ) -> impl IntoView {
            let style = "display:inline-block";
            $(
                let style = if $color {
                    concat!("display:inline-block;color:",stringify!($color),";")
                } else { style };
            )?
            make_icon($icon,style,width,height)
        }
    };
}

icon! {
    LinkIcon = icondata_bi::BiLinkRegular,
    FullscreenIcon = icondata_ai::AiFullscreenOutlined,
    CheckmarkIcon(green) = icondata_ai::AiCheckCircleOutlined,
    XMarkIcon(red) = icondata_ai::AiCloseCircleOutlined,
    OpenBookIcon = icondata_bi::BiBookContentRegular,
    ClosedBookIcon = icondata_bi::BiBookSolid,
    LibraryIcon = icondata_bi::BiLibraryRegular,
    FolderIcon = icondata_bi::BiFolderRegular,
    FileIcon = icondata_bi::BiFileRegular,
    PdfIcon = icondata_bs::BsFiletypePdf,
    SearchIcon = icondata_ai::AiSearchOutlined,
    VSCodeIcon = icondata_tb::TbBrandVscodeOutline,
    BurgerIcon = icondata_ch::ChMenuHamburger
}
