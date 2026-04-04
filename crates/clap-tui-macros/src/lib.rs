use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    Error, ExprPath, FnArg, GenericArgument, ItemFn, PatType, PathArguments, ReturnType, Token,
    Type, parse_macro_input,
};

/// Expand a `main` wrapper that delegates to `clap_tui::ParserLauncher`.
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MainArgs);
    let function = parse_macro_input!(input as ItemFn);

    match expand_main(args, function) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[derive(Default)]
struct MainArgs {
    config: Option<ExprPath>,
}

impl Parse for MainArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let ident = input.parse::<syn::Ident>()?;
        if ident != "config" {
            return Err(Error::new_spanned(
                ident,
                "unsupported argument; expected `config = path::to::fn`",
            ));
        }

        input.parse::<Token![=]>()?;
        let config = input.parse::<ExprPath>()?;

        if !input.is_empty() {
            return Err(input.error("unexpected extra tokens in `#[clap_tui::main(...)]`"));
        }

        Ok(Self {
            config: Some(config),
        })
    }
}

fn expand_main(args: MainArgs, function: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    validate_signature(&function)?;

    let attrs = &function.attrs;
    let vis = &function.vis;
    let sig = &function.sig;
    let block = &function.block;
    let function_name = &sig.ident;
    let runner_name = format_ident!("__clap_tui_user_{}", function_name);
    let input = sig.inputs.first().expect("validated parameter").clone();
    let parser_ty = function_input_type(&input)?;
    let output = &sig.output;
    let config_expr = args
        .config
        .map(|path| quote!(#path()))
        .unwrap_or_else(|| quote!(::clap_tui::TuiConfig::default()));

    Ok(quote! {
        #(#attrs)*
        fn #runner_name(#input) #output #block

        #vis fn #function_name() #output {
            ::clap_tui::ParserLauncher::<#parser_ty>::new()
                .with_config(#config_expr)
                .run(#runner_name)
        }
    })
}

fn validate_signature(function: &ItemFn) -> syn::Result<()> {
    let sig = &function.sig;

    if sig.ident != "main" {
        return Err(Error::new_spanned(
            &sig.ident,
            "`#[clap_tui::main]` only supports functions named `main`",
        ));
    }

    if sig.asyncness.is_some()
        || sig.constness.is_some()
        || sig.unsafety.is_some()
        || sig.abi.is_some()
        || !sig.generics.params.is_empty()
        || sig.variadic.is_some()
    {
        return Err(Error::new_spanned(
            sig,
            "`#[clap_tui::main]` requires a synchronous free function with no generics",
        ));
    }

    if sig.inputs.len() != 1 {
        return Err(Error::new_spanned(
            &sig.inputs,
            "`#[clap_tui::main]` requires exactly one typed parser parameter",
        ));
    }

    function_input_type(sig.inputs.first().expect("validated length"))?;
    validate_result_output(&sig.output)
}

fn function_input_type(input: &FnArg) -> syn::Result<&Type> {
    match input {
        FnArg::Typed(PatType { ty, .. }) => Ok(ty.as_ref()),
        FnArg::Receiver(receiver) => Err(Error::new_spanned(
            receiver,
            "`#[clap_tui::main]` does not support methods",
        )),
    }
}

fn validate_result_output(output: &ReturnType) -> syn::Result<()> {
    let ReturnType::Type(_, ty) = output else {
        return Err(Error::new_spanned(
            output,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    };

    let Type::Path(type_path) = ty.as_ref() else {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    };

    let Some(segment) = type_path.path.segments.last() else {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    };

    if segment.ident != "Result" {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    };

    let mut generic_arguments = arguments.args.iter();
    let Some(GenericArgument::Type(Type::Tuple(tuple))) = generic_arguments.next() else {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    };
    if !tuple.elems.is_empty() {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    }
    if !matches!(generic_arguments.next(), Some(GenericArgument::Type(_))) {
        return Err(Error::new_spanned(
            ty,
            "`#[clap_tui::main]` requires a `Result<(), E>` return type",
        ));
    }

    Ok(())
}
