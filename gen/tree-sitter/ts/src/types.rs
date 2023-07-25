use std::fmt::Display;

use hyper_ast::{
    store::defaults::NodeIdentifier,
    tree_gen::parser::NodeWithU16TypeId,
    types::{
        AnyType, HyperType, Lang, LangRef, NodeId, TypeStore, TypeTrait, TypedNodeId,
    },
};

#[cfg(feature = "legion")]
mod legion_impls {
    use super::*;

    use crate::TNode;

    impl<'a> TNode<'a> {
        pub fn obtain_type<T>(&self, _: &mut impl TsEnabledTypeStore<T>) -> Type {
            let t = self.kind_id();
            Type::from_u16(t)
        }
    }

    use hyper_ast::{store::nodes::legion::HashedNodeRef, types::TypeIndex};

    impl<'a, TS: TsEnabledTypeStore<HashedNodeRef<'a, Type>>> From<TS> for Single {
        fn from(value: TS) -> Self {
            Self {
                mask: TS::MASK,
                lang: TS::LANG,
            }
        }
    }

    impl<'a> TypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        type Ty = Type;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;
        fn resolve_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Ty {
            n.get_component::<Type>().unwrap().clone()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            From::<&'static (dyn LangRef<Type>)>::from(&Ts)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Ts),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
    }
    impl<'a> TsEnabledTypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        const LANG: TypeInternalSize = Self::Ts as u16;

        fn _intern(l: u16, t: u16) -> Self::Ty {
            // T((u16::MAX - l as u16) | t)
            todo!()
        }
        fn intern(&self, t: Type) -> Self::Ty {
            t
        }

        fn resolve(&self, t: Self::Ty) -> Type {
            t
            // let t = t.0 as u16;
            // let t = t & !TStore::MASK;
            // Type::resolve(t)
        }
    }
    impl<'a> TypeStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
        type Ty = AnyType;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;
        fn resolve_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Ty {
            From::<&'static (dyn HyperType)>::from(LangRef::<Type>::make(
                &Ts,
                *n.get_component::<Type>().unwrap() as u16,
            ))
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            From::<&'static (dyn LangRef<AnyType>)>::from(&Ts)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Ts),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
    }
}

pub trait TsEnabledTypeStore<T>: TypeStore<T> {
    const LANG: u16;
    fn intern(&self, t: Type) -> Self::Ty {
        let t = t as u16;
        Self::_intern(Self::LANG, t)
    }
    fn _intern(l: u16, t: u16) -> Self::Ty;
    fn resolve(&self, t: Self::Ty) -> Type;
    //  {
    //     todo!()
    //     // let t = t.0 as u16;
    //     // let t = t & !TStore::MASK;
    //     // Type::resolve(t)
    // }
}

#[cfg(feature = "legion")]
mod exp {
    use super::*;
    use crate::TNode;
    type TypeInternalSize = u16;

    struct T(TypeInternalSize);

    #[repr(u8)]
    enum TStore {
        Cpp = 0,
        Java = 1,
        Xml = 2,
        Ts = 3,
    }
    enum Types {
        Cpp(Type),
        Java(Type),
        Xml(Type),
        Ts(Type),
    }

    trait TypeStore {
        type T;
        const MASK: TypeInternalSize;
    }

    impl TypeStore for TStore {
        type T = T;
        const MASK: TypeInternalSize = 0b1100_0000_0000_0000;
    }

    impl TStore {
        pub fn intern_cpp(&self, n: TNode) -> T {
            let t = n.kind_id();
            Self::_intern(TStore::Cpp, t)
        }
        pub fn intern_java(&self, n: TNode) -> T {
            let t = n.kind_id();
            Self::_intern(TStore::Java, t)
        }
        pub fn intern_xml(&self, n: TNode) -> T {
            let t = n.kind_id();
            Self::_intern(TStore::Xml, t)
        }
        pub fn intern_ts(&self, n: TNode) -> T {
            let t = n.kind_id();
            let t = Type::from_u16(t);
            let t = t as u16;
            Self::_intern(TStore::Ts, t)
        }
        fn _intern(l: TStore, t: u16) -> T {
            T((u16::MAX - l as u16) | t)
        }
        fn resolve_ts_unchecked(&self, t: T) -> Type {
            let t = t.0 as u16;
            let t = t & !TStore::MASK;
            Type::resolve(t)
        }
        pub fn resolve(&self, t: T) -> Types {
            const TS: u16 = TStore::Ts as u16;
            const CPP: u16 = TStore::Cpp as u16;
            const JAVA: u16 = TStore::Java as u16;
            const XML: u16 = TStore::Xml as u16;
            match u16::MAX - (t.0 & TStore::MASK) {
                TS => Types::Ts(self.resolve_ts_unchecked(t)),
                JAVA => Types::Java(panic!()),
                CPP => Types::Cpp(panic!()),
                XML => Types::Xml(panic!()),
                x => panic!(),
            }
        }
    }

    #[test]
    fn f() {
        use std::mem::size_of;
        dbg!(size_of::<T>());
        dbg!(size_of::<u16>());
    }
}

impl Type {
    pub fn resolve(t: u16) -> Self {
        assert!(t < COUNT);
        unsafe { std::mem::transmute(t) }
    }
}

pub struct Single {
    mask: TypeInternalSize,
    lang: TypeInternalSize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TIdN<IdN>(IdN);

impl<IdN: Clone + Eq + NodeId> NodeId for TIdN<IdN> {
    type IdN = IdN;

    fn as_id(&self) -> &Self::IdN {
        &self.0
    }

    unsafe fn from_id(id: Self::IdN) -> Self {
        Self(id)
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        todo!()
    }
}

impl<IdN: Clone + Eq + NodeId> TypedNodeId for TIdN<IdN> {
    type Ty = Type;
}

#[repr(u8)]
pub(crate) enum TStore {
    Ts = 0,
}

impl Default for TStore {
    fn default() -> Self {
        Self::Ts
    }
}

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);

pub struct Ts;

impl Ts {
    const INST: Ts = Ts;
}

impl LangRef<AnyType> for Ts {
    fn make(&self, t: u16) -> &'static AnyType {
        panic!()
        // &From::<&'static dyn HyperType>::from(&S_T_L[t as usize])
    }
    fn to_u16(&self, t: AnyType) -> u16 {
        // t as u16
        let t = t.as_any().downcast_ref::<Type>().unwrap();
        *t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Ts>()
    }
}

impl LangRef<Type> for Ts {
    fn make(&self, t: u16) -> &'static Type {
        &S_T_L[t as usize]
    }
    fn to_u16(&self, t: Type) -> u16 {
        t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Ts>()
    }
}

impl Lang<Type> for Ts {
    fn make(t: u16) -> &'static Type {
        Ts.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Ts.to_u16(t)
    }
}

impl HyperType for Type {
    fn is_directory(&self) -> bool {
        self == &Type::Directory
    }

    fn is_file(&self) -> bool {
        todo!()
    }

    fn is_spaces(&self) -> bool {
        self == &Type::Spaces
        // setting TS0 as space is causing an issue with global_pos_with_spaces
        // and TS0 is end list of tokens, so maybe other issues.
        // Actual fix is to skip TS0 in skipable_pre in the generator,
        // thus TSO should not appear anymore in generated ast.
        // || self == &Type::TS0
    }

    fn is_syntax(&self) -> bool {
        todo!()
    }

    fn as_shared(&self) -> hyper_ast::types::Shared {
        use hyper_ast::types::Shared;

        match self {
            Type::ClassDeclaration => Shared::TypeDeclaration,
            Type::EnumDeclaration => Shared::TypeDeclaration,
            Type::TypeAliasDeclaration => Shared::TypeDeclaration,
            Type::InterfaceDeclaration => Shared::TypeDeclaration,
            Type::AbstractClassDeclaration => Shared::TypeDeclaration,
            Type::FunctionDeclaration => Shared::TypeDeclaration,
            Type::Comment => Shared::Comment,
            Type::Identifier => Shared::Identifier,
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_lang(&self) -> hyper_ast::types::LangWrapper<Self>
    where
        Self: Sized,
    {
        From::<&'static (dyn LangRef<Self>)>::from(&Ts)
    }
}
impl TypeTrait for Type {
    type Lang = Ts;

    fn is_fork(&self) -> bool {
        todo!()
    }

    fn is_literal(&self) -> bool {
        todo!()
    }

    fn is_primitive(&self) -> bool {
        todo!()
    }

    fn is_type_declaration(&self) -> bool {
        todo!()
    }

    fn is_identifier(&self) -> bool {
        todo!()
    }

    fn is_instance_ref(&self) -> bool {
        todo!()
    }

    fn is_type_body(&self) -> bool {
        todo!()
    }

    fn is_value_member(&self) -> bool {
        todo!()
    }

    fn is_executable_member(&self) -> bool {
        todo!()
    }

    fn is_statement(&self) -> bool {
        todo!()
    }

    fn is_declarative_statement(&self) -> bool {
        todo!()
    }

    fn is_structural_statement(&self) -> bool {
        todo!()
    }

    fn is_block_related(&self) -> bool {
        todo!()
    }

    fn is_simple_statement(&self) -> bool {
        todo!()
    }

    fn is_local_declare(&self) -> bool {
        todo!()
    }

    fn is_parameter(&self) -> bool {
        todo!()
    }

    fn is_parameter_list(&self) -> bool {
        todo!()
    }

    fn is_argument_list(&self) -> bool {
        todo!()
    }

    fn is_expression(&self) -> bool {
        todo!()
    }

    fn is_comment(&self) -> bool {
        todo!()
    }
}

// 356 + directory  + spaces
const COUNT: u16 = 358;

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[repr(u16)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
    End,
    Identifier,
    HashBangLine,
    Export,
    Star,
    Default,
    Type,
    Eq,
    As,
    Namespace,
    LBrace,
    Comma,
    RBrace,
    Typeof,
    Import,
    From,
    Var,
    Let,
    Const,
    Bang,
    Else,
    If,
    Switch,
    For,
    LParen,
    RParen,
    Await,
    In,
    Of,
    While,
    Do,
    Try,
    With,
    Break,
    Continue,
    Debugger,
    Return,
    Throw,
    SemiColon,
    Colon,
    Case,
    Catch,
    Finally,
    Yield,
    LBracket,
    RBracket,
    LT,
    GT,
    Slash,
    Dot,
    Class,
    Async,
    Function,
    TS0,
    TS1,
    New,
    PlusEq,
    DashEq,
    StarEq,
    SlashEq,
    PercentEq,
    CaretEq,
    AmpEq,
    PipeEq,
    GtGtEq,
    GtGtGtEq,
    LtLtEq,
    TS2,
    TS3,
    TS4,
    TS5,
    DotDotDot,
    AmpAmp,
    PipePipe,
    GtGt,
    GtGtGt,
    LtLt,
    Amp,
    Caret,
    Pipe,
    Plus,
    Dash,
    Percent,
    TS6,
    LTEq,
    EqEq,
    TS7,
    BangEq,
    TS8,
    GTEq,
    TS9,
    Instanceof,
    Tilde,
    Void,
    Delete,
    PlusPlus,
    DashDash,
    TS10,
    TS11,
    StringFragment,
    EscapeSequence,
    Comment,
    TS12,
    TS13,
    RegexPattern,
    RegexFlags,
    Number,
    PrivatePropertyIdentifier,
    Target,
    This,
    Super,
    True,
    False,
    Null,
    Undefined,
    At,
    Static,
    Readonly,
    Get,
    Set,
    QMark,
    Declare,
    Public,
    Private,
    Protected,
    Override,
    Module,
    Any,
    Boolean,
    String,
    Symbol,
    Abstract,
    Require,
    Extends,
    Implements,
    Global,
    Interface,
    Enum,
    TS14,
    TS15,
    Asserts,
    Infer,
    Is,
    Keyof,
    Unknown,
    Never,
    Object,
    TS16,
    TS17,
    TS18,
    TS19,
    TS20,
    Program,
    ExportStatement,
    ExportClause,
    ExportSpecifier,
    Declaration,
    ImportStatement,
    ImportClause,
    TS21,
    NamespaceImport,
    NamedImports,
    ExpressionStatement,
    VariableDeclaration,
    LexicalDeclaration,
    VariableDeclarator,
    StatementBlock,
    ElseClause,
    IfStatement,
    SwitchStatement,
    ForStatement,
    ForInStatement,
    TS22,
    WhileStatement,
    DoStatement,
    TryStatement,
    WithStatement,
    BreakStatement,
    ContinueStatement,
    DebuggerStatement,
    ReturnStatement,
    ThrowStatement,
    EmptyStatement,
    LabeledStatement,
    SwitchBody,
    SwitchCase,
    SwitchDefault,
    CatchClause,
    FinallyClause,
    ParenthesizedExpression,
    Expression,
    PrimaryExpression,
    YieldExpression,
    ObjectPattern,
    AssignmentPattern,
    ObjectAssignmentPattern,
    Array,
    ArrayPattern,
    NestedIdentifier,
    ClassDeclaration,
    ClassHeritage,
    FunctionDeclaration,
    GeneratorFunction,
    GeneratorFunctionDeclaration,
    ArrowFunction,
    TS23,
    TS24,
    CallExpression,
    NewExpression,
    AwaitExpression,
    MemberExpression,
    SubscriptExpression,
    AssignmentExpression,
    TS25,
    AugmentedAssignmentExpression,
    TS26,
    TS27,
    SpreadElement,
    TernaryExpression,
    BinaryExpression,
    UnaryExpression,
    UpdateExpression,
    SequenceExpression,
    TemplateString,
    TemplateSubstitution,
    Regex,
    MetaProperty,
    Arguments,
    Decorator,
    ClassBody,
    FormalParameters,
    Pattern,
    RestPattern,
    MethodDefinition,
    Pair,
    PairPattern,
    TS28,
    ComputedPropertyName,
    PublicFieldDefinition,
    NonNullExpression,
    MethodSignature,
    AbstractMethodSignature,
    FunctionSignature,
    TypeAssertion,
    AsExpression,
    ImportRequireClause,
    ExtendsClause,
    ImplementsClause,
    AmbientDeclaration,
    AbstractClassDeclaration,
    InternalModule,
    TS29,
    ImportAlias,
    NestedTypeIdentifier,
    InterfaceDeclaration,
    ExtendsTypeClause,
    EnumDeclaration,
    EnumBody,
    EnumAssignment,
    TypeAliasDeclaration,
    AccessibilityModifier,
    OverrideModifier,
    RequiredParameter,
    OptionalParameter,
    TS30,
    OmittingTypeAnnotation,
    OptingTypeAnnotation,
    TypeAnnotation,
    TS31,
    OptionalType,
    RestType,
    TS32,
    ConstructorType,
    PrimaryType,
    TemplateType,
    TemplateLiteralType,
    InferType,
    ConditionalType,
    GenericType,
    TypePredicate,
    TypePredicateAnnotation,
    TypeQuery,
    IndexTypeQuery,
    LookupType,
    MappedTypeClause,
    LiteralType,
    ExistentialType,
    FlowMaybeType,
    ParenthesizedType,
    PredefinedType,
    TypeArguments,
    ObjectType,
    CallSignature,
    PropertySignature,
    TypeParameters,
    TypeParameter,
    DefaultType,
    Constraint,
    ConstructSignature,
    IndexSignature,
    ArrayType,
    TupleType,
    ReadonlyType,
    UnionType,
    IntersectionType,
    FunctionType,
    ProgramRepeat1,
    ExportStatementRepeat1,
    ExportClauseRepeat1,
    NamedImportsRepeat1,
    VariableDeclarationRepeat1,
    SwitchBodyRepeat1,
    ObjectRepeat1,
    ObjectPatternRepeat1,
    ArrayRepeat1,
    ArrayPatternRepeat1,
    StringRepeat1,
    StringRepeat2,
    TemplateStringRepeat1,
    ClassBodyRepeat1,
    FormalParametersRepeat1,
    ExtendsClauseRepeat1,
    ImplementsClauseRepeat1,
    ExtendsTypeClauseRepeat1,
    EnumBodyRepeat1,
    TemplateLiteralTypeRepeat1,
    ObjectTypeRepeat1,
    TypeParametersRepeat1,
    TupleTypeRepeat1,
    ImportSpecifier,
    NamespaceExport,
    PropertyIdentifier,
    ShorthandPropertyIdentifier,
    ShorthandPropertyIdentifierPattern,
    StatementIdentifier,
    ThisType,
    TypeIdentifier,
    Spaces,
    Directory,
    ERROR,
}
impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::Identifier,
            2u16 => Type::HashBangLine,
            3u16 => Type::Export,
            4u16 => Type::Star,
            5u16 => Type::Default,
            6u16 => Type::Type,
            7u16 => Type::Eq,
            8u16 => Type::As,
            9u16 => Type::Namespace,
            10u16 => Type::LBrace,
            11u16 => Type::Comma,
            12u16 => Type::RBrace,
            13u16 => Type::Typeof,
            14u16 => Type::Import,
            15u16 => Type::From,
            16u16 => Type::Var,
            17u16 => Type::Let,
            18u16 => Type::Const,
            19u16 => Type::Bang,
            20u16 => Type::Else,
            21u16 => Type::If,
            22u16 => Type::Switch,
            23u16 => Type::For,
            24u16 => Type::LParen,
            25u16 => Type::RParen,
            26u16 => Type::Await,
            27u16 => Type::In,
            28u16 => Type::Of,
            29u16 => Type::While,
            30u16 => Type::Do,
            31u16 => Type::Try,
            32u16 => Type::With,
            33u16 => Type::Break,
            34u16 => Type::Continue,
            35u16 => Type::Debugger,
            36u16 => Type::Return,
            37u16 => Type::Throw,
            38u16 => Type::SemiColon,
            39u16 => Type::Colon,
            40u16 => Type::Case,
            41u16 => Type::Catch,
            42u16 => Type::Finally,
            43u16 => Type::Yield,
            44u16 => Type::LBracket,
            45u16 => Type::RBracket,
            46u16 => Type::LT,
            47u16 => Type::GT,
            48u16 => Type::Slash,
            49u16 => Type::Dot,
            50u16 => Type::Class,
            51u16 => Type::Async,
            52u16 => Type::Function,
            53u16 => Type::TS0,
            54u16 => Type::TS1,
            55u16 => Type::New,
            56u16 => Type::PlusEq,
            57u16 => Type::DashEq,
            58u16 => Type::StarEq,
            59u16 => Type::SlashEq,
            60u16 => Type::PercentEq,
            61u16 => Type::CaretEq,
            62u16 => Type::AmpEq,
            63u16 => Type::PipeEq,
            64u16 => Type::GtGtEq,
            65u16 => Type::GtGtGtEq,
            66u16 => Type::LtLtEq,
            67u16 => Type::TS2,
            68u16 => Type::TS3,
            69u16 => Type::TS4,
            70u16 => Type::TS5,
            71u16 => Type::DotDotDot,
            72u16 => Type::AmpAmp,
            73u16 => Type::PipePipe,
            74u16 => Type::GtGt,
            75u16 => Type::GtGtGt,
            76u16 => Type::LtLt,
            77u16 => Type::Amp,
            78u16 => Type::Caret,
            79u16 => Type::Pipe,
            80u16 => Type::Plus,
            81u16 => Type::Dash,
            82u16 => Type::Percent,
            83u16 => Type::TS6,
            84u16 => Type::LTEq,
            85u16 => Type::EqEq,
            86u16 => Type::TS7,
            87u16 => Type::BangEq,
            88u16 => Type::TS8,
            89u16 => Type::GTEq,
            90u16 => Type::TS9,
            91u16 => Type::Instanceof,
            92u16 => Type::Tilde,
            93u16 => Type::Void,
            94u16 => Type::Delete,
            95u16 => Type::PlusPlus,
            96u16 => Type::DashDash,
            97u16 => Type::TS10,
            98u16 => Type::TS11,
            99u16 => Type::StringFragment,
            100u16 => Type::StringFragment,
            101u16 => Type::EscapeSequence,
            102u16 => Type::Comment,
            103u16 => Type::TS12,
            104u16 => Type::TS13,
            105u16 => Type::Slash,
            106u16 => Type::RegexPattern,
            107u16 => Type::RegexFlags,
            108u16 => Type::Number,
            109u16 => Type::PrivatePropertyIdentifier,
            110u16 => Type::Target,
            111u16 => Type::This,
            112u16 => Type::Super,
            113u16 => Type::True,
            114u16 => Type::False,
            115u16 => Type::Null,
            116u16 => Type::Undefined,
            117u16 => Type::At,
            118u16 => Type::Static,
            119u16 => Type::Readonly,
            120u16 => Type::Get,
            121u16 => Type::Set,
            122u16 => Type::QMark,
            123u16 => Type::Declare,
            124u16 => Type::Public,
            125u16 => Type::Private,
            126u16 => Type::Protected,
            127u16 => Type::Override,
            128u16 => Type::Module,
            129u16 => Type::Any,
            130u16 => Type::Number,
            131u16 => Type::Boolean,
            132u16 => Type::String,
            133u16 => Type::Symbol,
            134u16 => Type::Abstract,
            135u16 => Type::Require,
            136u16 => Type::Extends,
            137u16 => Type::Implements,
            138u16 => Type::Global,
            139u16 => Type::Interface,
            140u16 => Type::Enum,
            141u16 => Type::TS14,
            142u16 => Type::TS15,
            143u16 => Type::Asserts,
            144u16 => Type::Infer,
            145u16 => Type::Is,
            146u16 => Type::Keyof,
            147u16 => Type::Unknown,
            148u16 => Type::Never,
            149u16 => Type::Object,
            150u16 => Type::TS16,
            151u16 => Type::TS17,
            152u16 => Type::TS18,
            153u16 => Type::TS19,
            154u16 => Type::QMark,
            155u16 => Type::TS20,
            156u16 => Type::Program,
            157u16 => Type::ExportStatement,
            158u16 => Type::ExportClause,
            159u16 => Type::ExportSpecifier,
            160u16 => Type::Declaration,
            161u16 => Type::Import,
            162u16 => Type::ImportStatement,
            163u16 => Type::ImportClause,
            164u16 => Type::TS21,
            165u16 => Type::NamespaceImport,
            166u16 => Type::NamedImports,
            167u16 => Type::ExpressionStatement,
            168u16 => Type::VariableDeclaration,
            169u16 => Type::LexicalDeclaration,
            170u16 => Type::VariableDeclarator,
            171u16 => Type::StatementBlock,
            172u16 => Type::ElseClause,
            173u16 => Type::IfStatement,
            174u16 => Type::SwitchStatement,
            175u16 => Type::ForStatement,
            176u16 => Type::ForInStatement,
            177u16 => Type::TS22,
            178u16 => Type::WhileStatement,
            179u16 => Type::DoStatement,
            180u16 => Type::TryStatement,
            181u16 => Type::WithStatement,
            182u16 => Type::BreakStatement,
            183u16 => Type::ContinueStatement,
            184u16 => Type::DebuggerStatement,
            185u16 => Type::ReturnStatement,
            186u16 => Type::ThrowStatement,
            187u16 => Type::EmptyStatement,
            188u16 => Type::LabeledStatement,
            189u16 => Type::SwitchBody,
            190u16 => Type::SwitchCase,
            191u16 => Type::SwitchDefault,
            192u16 => Type::CatchClause,
            193u16 => Type::FinallyClause,
            194u16 => Type::ParenthesizedExpression,
            195u16 => Type::Expression,
            196u16 => Type::PrimaryExpression,
            197u16 => Type::YieldExpression,
            198u16 => Type::Object,
            199u16 => Type::ObjectPattern,
            200u16 => Type::AssignmentPattern,
            201u16 => Type::ObjectAssignmentPattern,
            202u16 => Type::Array,
            203u16 => Type::ArrayPattern,
            204u16 => Type::NestedIdentifier,
            205u16 => Type::Class,
            206u16 => Type::ClassDeclaration,
            207u16 => Type::ClassHeritage,
            208u16 => Type::Function,
            209u16 => Type::FunctionDeclaration,
            210u16 => Type::GeneratorFunction,
            211u16 => Type::GeneratorFunctionDeclaration,
            212u16 => Type::ArrowFunction,
            213u16 => Type::TS23,
            214u16 => Type::TS24,
            215u16 => Type::CallExpression,
            216u16 => Type::NewExpression,
            217u16 => Type::AwaitExpression,
            218u16 => Type::MemberExpression,
            219u16 => Type::SubscriptExpression,
            220u16 => Type::AssignmentExpression,
            221u16 => Type::TS25,
            222u16 => Type::AugmentedAssignmentExpression,
            223u16 => Type::TS26,
            224u16 => Type::TS27,
            225u16 => Type::SpreadElement,
            226u16 => Type::TernaryExpression,
            227u16 => Type::BinaryExpression,
            228u16 => Type::UnaryExpression,
            229u16 => Type::UpdateExpression,
            230u16 => Type::SequenceExpression,
            231u16 => Type::String,
            232u16 => Type::TemplateString,
            233u16 => Type::TemplateSubstitution,
            234u16 => Type::Regex,
            235u16 => Type::MetaProperty,
            236u16 => Type::Arguments,
            237u16 => Type::Decorator,
            238u16 => Type::MemberExpression,
            239u16 => Type::CallExpression,
            240u16 => Type::ClassBody,
            241u16 => Type::FormalParameters,
            242u16 => Type::Pattern,
            243u16 => Type::RestPattern,
            244u16 => Type::MethodDefinition,
            245u16 => Type::Pair,
            246u16 => Type::PairPattern,
            247u16 => Type::TS28,
            248u16 => Type::ComputedPropertyName,
            249u16 => Type::PublicFieldDefinition,
            250u16 => Type::NonNullExpression,
            251u16 => Type::MethodSignature,
            252u16 => Type::AbstractMethodSignature,
            253u16 => Type::FunctionSignature,
            254u16 => Type::TypeAssertion,
            255u16 => Type::AsExpression,
            256u16 => Type::ImportRequireClause,
            257u16 => Type::ExtendsClause,
            258u16 => Type::ImplementsClause,
            259u16 => Type::AmbientDeclaration,
            260u16 => Type::AbstractClassDeclaration,
            261u16 => Type::Module,
            262u16 => Type::InternalModule,
            263u16 => Type::TS29,
            264u16 => Type::ImportAlias,
            265u16 => Type::NestedTypeIdentifier,
            266u16 => Type::InterfaceDeclaration,
            267u16 => Type::ExtendsTypeClause,
            268u16 => Type::EnumDeclaration,
            269u16 => Type::EnumBody,
            270u16 => Type::EnumAssignment,
            271u16 => Type::TypeAliasDeclaration,
            272u16 => Type::AccessibilityModifier,
            273u16 => Type::OverrideModifier,
            274u16 => Type::RequiredParameter,
            275u16 => Type::OptionalParameter,
            276u16 => Type::TS30,
            277u16 => Type::OmittingTypeAnnotation,
            278u16 => Type::OptingTypeAnnotation,
            279u16 => Type::TypeAnnotation,
            280u16 => Type::Asserts,
            281u16 => Type::TS31,
            282u16 => Type::RequiredParameter,
            283u16 => Type::OptionalParameter,
            284u16 => Type::OptionalType,
            285u16 => Type::RestType,
            286u16 => Type::TS32,
            287u16 => Type::ConstructorType,
            288u16 => Type::PrimaryType,
            289u16 => Type::TemplateType,
            290u16 => Type::TemplateLiteralType,
            291u16 => Type::InferType,
            292u16 => Type::ConditionalType,
            293u16 => Type::GenericType,
            294u16 => Type::TypePredicate,
            295u16 => Type::TypePredicateAnnotation,
            296u16 => Type::MemberExpression,
            297u16 => Type::SubscriptExpression,
            298u16 => Type::CallExpression,
            299u16 => Type::TypeQuery,
            300u16 => Type::IndexTypeQuery,
            301u16 => Type::LookupType,
            302u16 => Type::MappedTypeClause,
            303u16 => Type::LiteralType,
            304u16 => Type::UnaryExpression,
            305u16 => Type::ExistentialType,
            306u16 => Type::FlowMaybeType,
            307u16 => Type::ParenthesizedType,
            308u16 => Type::PredefinedType,
            309u16 => Type::TypeArguments,
            310u16 => Type::ObjectType,
            311u16 => Type::CallSignature,
            312u16 => Type::PropertySignature,
            313u16 => Type::TypeParameters,
            314u16 => Type::TypeParameter,
            315u16 => Type::DefaultType,
            316u16 => Type::Constraint,
            317u16 => Type::ConstructSignature,
            318u16 => Type::IndexSignature,
            319u16 => Type::ArrayType,
            320u16 => Type::TupleType,
            321u16 => Type::ReadonlyType,
            322u16 => Type::UnionType,
            323u16 => Type::IntersectionType,
            324u16 => Type::FunctionType,
            325u16 => Type::ProgramRepeat1,
            326u16 => Type::ExportStatementRepeat1,
            327u16 => Type::ExportClauseRepeat1,
            328u16 => Type::NamedImportsRepeat1,
            329u16 => Type::VariableDeclarationRepeat1,
            330u16 => Type::SwitchBodyRepeat1,
            331u16 => Type::ObjectRepeat1,
            332u16 => Type::ObjectPatternRepeat1,
            333u16 => Type::ArrayRepeat1,
            334u16 => Type::ArrayPatternRepeat1,
            335u16 => Type::StringRepeat1,
            336u16 => Type::StringRepeat2,
            337u16 => Type::TemplateStringRepeat1,
            338u16 => Type::ClassBodyRepeat1,
            339u16 => Type::FormalParametersRepeat1,
            340u16 => Type::ExtendsClauseRepeat1,
            341u16 => Type::ImplementsClauseRepeat1,
            342u16 => Type::ExtendsTypeClauseRepeat1,
            343u16 => Type::EnumBodyRepeat1,
            344u16 => Type::TemplateLiteralTypeRepeat1,
            345u16 => Type::ObjectTypeRepeat1,
            346u16 => Type::TypeParametersRepeat1,
            347u16 => Type::TupleTypeRepeat1,
            348u16 => Type::ImportSpecifier,
            349u16 => Type::NamespaceExport,
            350u16 => Type::PropertyIdentifier,
            351u16 => Type::ShorthandPropertyIdentifier,
            352u16 => Type::ShorthandPropertyIdentifierPattern,
            353u16 => Type::StatementIdentifier,
            354u16 => Type::ThisType,
            355u16 => Type::TypeIdentifier,
            356u16 => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(&self) -> Option<Type> {
        todo!()
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Identifier => "identifier",
            Type::HashBangLine => "hash_bang_line",
            Type::Export => "export",
            Type::Star => "*",
            Type::Default => "default",
            Type::Type => "type",
            Type::Eq => "=",
            Type::As => "as",
            Type::Namespace => "namespace",
            Type::LBrace => "{",
            Type::Comma => ",",
            Type::RBrace => "}",
            Type::Typeof => "typeof",
            Type::Import => "import",
            Type::From => "from",
            Type::Var => "var",
            Type::Let => "let",
            Type::Const => "const",
            Type::Bang => "!",
            Type::Else => "else",
            Type::If => "if",
            Type::Switch => "switch",
            Type::For => "for",
            Type::LParen => "(",
            Type::RParen => ")",
            Type::Await => "await",
            Type::In => "in",
            Type::Of => "of",
            Type::While => "while",
            Type::Do => "do",
            Type::Try => "try",
            Type::With => "with",
            Type::Break => "break",
            Type::Continue => "continue",
            Type::Debugger => "debugger",
            Type::Return => "return",
            Type::Throw => "throw",
            Type::SemiColon => ";",
            Type::Colon => ":",
            Type::Case => "case",
            Type::Catch => "catch",
            Type::Finally => "finally",
            Type::Yield => "yield",
            Type::LBracket => "[",
            Type::RBracket => "]",
            Type::LT => "<",
            Type::GT => ">",
            Type::Slash => "/",
            Type::Dot => ".",
            Type::Class => "class",
            Type::Async => "async",
            Type::Function => "function",
            Type::TS0 => "=>",
            Type::TS1 => "?.",
            Type::New => "new",
            Type::PlusEq => "+=",
            Type::DashEq => "-=",
            Type::StarEq => "*=",
            Type::SlashEq => "/=",
            Type::PercentEq => "%=",
            Type::CaretEq => "^=",
            Type::AmpEq => "&=",
            Type::PipeEq => "|=",
            Type::GtGtEq => ">>=",
            Type::GtGtGtEq => ">>>=",
            Type::LtLtEq => "<<=",
            Type::TS2 => "**=",
            Type::TS3 => "&&=",
            Type::TS4 => "||=",
            Type::TS5 => "??=",
            Type::DotDotDot => "...",
            Type::AmpAmp => "&&",
            Type::PipePipe => "||",
            Type::GtGt => ">>",
            Type::GtGtGt => ">>>",
            Type::LtLt => "<<",
            Type::Amp => "&",
            Type::Caret => "^",
            Type::Pipe => "|",
            Type::Plus => "+",
            Type::Dash => "-",
            Type::Percent => "%",
            Type::TS6 => "**",
            Type::LTEq => "<=",
            Type::EqEq => "==",
            Type::TS7 => "===",
            Type::BangEq => "!=",
            Type::TS8 => "!==",
            Type::GTEq => ">=",
            Type::TS9 => "??",
            Type::Instanceof => "instanceof",
            Type::Tilde => "~",
            Type::Void => "void",
            Type::Delete => "delete",
            Type::PlusPlus => "++",
            Type::DashDash => "--",
            Type::TS10 => "\"",
            Type::TS11 => "'",
            Type::StringFragment => "string_fragment",
            Type::EscapeSequence => "escape_sequence",
            Type::Comment => "comment",
            Type::TS12 => "`",
            Type::TS13 => "${",
            Type::RegexPattern => "regex_pattern",
            Type::RegexFlags => "regex_flags",
            Type::Number => "number",
            Type::PrivatePropertyIdentifier => "private_property_identifier",
            Type::Target => "target",
            Type::This => "this",
            Type::Super => "super",
            Type::True => "true",
            Type::False => "false",
            Type::Null => "null",
            Type::Undefined => "undefined",
            Type::At => "@",
            Type::Static => "static",
            Type::Readonly => "readonly",
            Type::Get => "get",
            Type::Set => "set",
            Type::QMark => "?",
            Type::Declare => "declare",
            Type::Public => "public",
            Type::Private => "private",
            Type::Protected => "protected",
            Type::Override => "override",
            Type::Module => "module",
            Type::Any => "any",
            Type::Boolean => "boolean",
            Type::String => "string",
            Type::Symbol => "symbol",
            Type::Abstract => "abstract",
            Type::Require => "require",
            Type::Extends => "extends",
            Type::Implements => "implements",
            Type::Global => "global",
            Type::Interface => "interface",
            Type::Enum => "enum",
            Type::TS14 => "-?:",
            Type::TS15 => "?:",
            Type::Asserts => "asserts",
            Type::Infer => "infer",
            Type::Is => "is",
            Type::Keyof => "keyof",
            Type::Unknown => "unknown",
            Type::Never => "never",
            Type::Object => "object",
            Type::TS16 => "{|",
            Type::TS17 => "|}",
            Type::TS18 => "_automatic_semicolon",
            Type::TS19 => "_template_chars",
            Type::TS20 => "_function_signature_automatic_semicolon",
            Type::Program => "program",
            Type::ExportStatement => "export_statement",
            Type::ExportClause => "export_clause",
            Type::ExportSpecifier => "export_specifier",
            Type::Declaration => "declaration",
            Type::ImportStatement => "import_statement",
            Type::ImportClause => "import_clause",
            Type::TS21 => "_from_clause",
            Type::NamespaceImport => "namespace_import",
            Type::NamedImports => "named_imports",
            Type::ExpressionStatement => "expression_statement",
            Type::VariableDeclaration => "variable_declaration",
            Type::LexicalDeclaration => "lexical_declaration",
            Type::VariableDeclarator => "variable_declarator",
            Type::StatementBlock => "statement_block",
            Type::ElseClause => "else_clause",
            Type::IfStatement => "if_statement",
            Type::SwitchStatement => "switch_statement",
            Type::ForStatement => "for_statement",
            Type::ForInStatement => "for_in_statement",
            Type::TS22 => "_for_header",
            Type::WhileStatement => "while_statement",
            Type::DoStatement => "do_statement",
            Type::TryStatement => "try_statement",
            Type::WithStatement => "with_statement",
            Type::BreakStatement => "break_statement",
            Type::ContinueStatement => "continue_statement",
            Type::DebuggerStatement => "debugger_statement",
            Type::ReturnStatement => "return_statement",
            Type::ThrowStatement => "throw_statement",
            Type::EmptyStatement => "empty_statement",
            Type::LabeledStatement => "labeled_statement",
            Type::SwitchBody => "switch_body",
            Type::SwitchCase => "switch_case",
            Type::SwitchDefault => "switch_default",
            Type::CatchClause => "catch_clause",
            Type::FinallyClause => "finally_clause",
            Type::ParenthesizedExpression => "parenthesized_expression",
            Type::Expression => "expression",
            Type::PrimaryExpression => "primary_expression",
            Type::YieldExpression => "yield_expression",
            Type::ObjectPattern => "object_pattern",
            Type::AssignmentPattern => "assignment_pattern",
            Type::ObjectAssignmentPattern => "object_assignment_pattern",
            Type::Array => "array",
            Type::ArrayPattern => "array_pattern",
            Type::NestedIdentifier => "nested_identifier",
            Type::ClassDeclaration => "class_declaration",
            Type::ClassHeritage => "class_heritage",
            Type::FunctionDeclaration => "function_declaration",
            Type::GeneratorFunction => "generator_function",
            Type::GeneratorFunctionDeclaration => "generator_function_declaration",
            Type::ArrowFunction => "arrow_function",
            Type::TS23 => "_call_signature",
            Type::TS24 => "_formal_parameter",
            Type::CallExpression => "call_expression",
            Type::NewExpression => "new_expression",
            Type::AwaitExpression => "await_expression",
            Type::MemberExpression => "member_expression",
            Type::SubscriptExpression => "subscript_expression",
            Type::AssignmentExpression => "assignment_expression",
            Type::TS25 => "_augmented_assignment_lhs",
            Type::AugmentedAssignmentExpression => "augmented_assignment_expression",
            Type::TS26 => "_initializer",
            Type::TS27 => "_destructuring_pattern",
            Type::SpreadElement => "spread_element",
            Type::TernaryExpression => "ternary_expression",
            Type::BinaryExpression => "binary_expression",
            Type::UnaryExpression => "unary_expression",
            Type::UpdateExpression => "update_expression",
            Type::SequenceExpression => "sequence_expression",
            Type::TemplateString => "template_string",
            Type::TemplateSubstitution => "template_substitution",
            Type::Regex => "regex",
            Type::MetaProperty => "meta_property",
            Type::Arguments => "arguments",
            Type::Decorator => "decorator",
            Type::ClassBody => "class_body",
            Type::FormalParameters => "formal_parameters",
            Type::Pattern => "pattern",
            Type::RestPattern => "rest_pattern",
            Type::MethodDefinition => "method_definition",
            Type::Pair => "pair",
            Type::PairPattern => "pair_pattern",
            Type::TS28 => "_property_name",
            Type::ComputedPropertyName => "computed_property_name",
            Type::PublicFieldDefinition => "public_field_definition",
            Type::NonNullExpression => "non_null_expression",
            Type::MethodSignature => "method_signature",
            Type::AbstractMethodSignature => "abstract_method_signature",
            Type::FunctionSignature => "function_signature",
            Type::TypeAssertion => "type_assertion",
            Type::AsExpression => "as_expression",
            Type::ImportRequireClause => "import_require_clause",
            Type::ExtendsClause => "extends_clause",
            Type::ImplementsClause => "implements_clause",
            Type::AmbientDeclaration => "ambient_declaration",
            Type::AbstractClassDeclaration => "abstract_class_declaration",
            Type::InternalModule => "internal_module",
            Type::TS29 => "_module",
            Type::ImportAlias => "import_alias",
            Type::NestedTypeIdentifier => "nested_type_identifier",
            Type::InterfaceDeclaration => "interface_declaration",
            Type::ExtendsTypeClause => "extends_type_clause",
            Type::EnumDeclaration => "enum_declaration",
            Type::EnumBody => "enum_body",
            Type::EnumAssignment => "enum_assignment",
            Type::TypeAliasDeclaration => "type_alias_declaration",
            Type::AccessibilityModifier => "accessibility_modifier",
            Type::OverrideModifier => "override_modifier",
            Type::RequiredParameter => "required_parameter",
            Type::OptionalParameter => "optional_parameter",
            Type::TS30 => "_parameter_name",
            Type::OmittingTypeAnnotation => "omitting_type_annotation",
            Type::OptingTypeAnnotation => "opting_type_annotation",
            Type::TypeAnnotation => "type_annotation",
            Type::TS31 => "_type",
            Type::OptionalType => "optional_type",
            Type::RestType => "rest_type",
            Type::TS32 => "_tuple_type_member",
            Type::ConstructorType => "constructor_type",
            Type::PrimaryType => "_primary_type",
            Type::TemplateType => "template_type",
            Type::TemplateLiteralType => "template_literal_type",
            Type::InferType => "infer_type",
            Type::ConditionalType => "conditional_type",
            Type::GenericType => "generic_type",
            Type::TypePredicate => "type_predicate",
            Type::TypePredicateAnnotation => "type_predicate_annotation",
            Type::TypeQuery => "type_query",
            Type::IndexTypeQuery => "index_type_query",
            Type::LookupType => "lookup_type",
            Type::MappedTypeClause => "mapped_type_clause",
            Type::LiteralType => "literal_type",
            Type::ExistentialType => "existential_type",
            Type::FlowMaybeType => "flow_maybe_type",
            Type::ParenthesizedType => "parenthesized_type",
            Type::PredefinedType => "predefined_type",
            Type::TypeArguments => "type_arguments",
            Type::ObjectType => "object_type",
            Type::CallSignature => "call_signature",
            Type::PropertySignature => "property_signature",
            Type::TypeParameters => "type_parameters",
            Type::TypeParameter => "type_parameter",
            Type::DefaultType => "default_type",
            Type::Constraint => "constraint",
            Type::ConstructSignature => "construct_signature",
            Type::IndexSignature => "index_signature",
            Type::ArrayType => "array_type",
            Type::TupleType => "tuple_type",
            Type::ReadonlyType => "readonly_type",
            Type::UnionType => "union_type",
            Type::IntersectionType => "intersection_type",
            Type::FunctionType => "function_type",
            Type::ProgramRepeat1 => "program_repeat1",
            Type::ExportStatementRepeat1 => "export_statement_repeat1",
            Type::ExportClauseRepeat1 => "export_clause_repeat1",
            Type::NamedImportsRepeat1 => "named_imports_repeat1",
            Type::VariableDeclarationRepeat1 => "variable_declaration_repeat1",
            Type::SwitchBodyRepeat1 => "switch_body_repeat1",
            Type::ObjectRepeat1 => "object_repeat1",
            Type::ObjectPatternRepeat1 => "object_pattern_repeat1",
            Type::ArrayRepeat1 => "array_repeat1",
            Type::ArrayPatternRepeat1 => "array_pattern_repeat1",
            Type::StringRepeat1 => "string_repeat1",
            Type::StringRepeat2 => "string_repeat2",
            Type::TemplateStringRepeat1 => "template_string_repeat1",
            Type::ClassBodyRepeat1 => "class_body_repeat1",
            Type::FormalParametersRepeat1 => "formal_parameters_repeat1",
            Type::ExtendsClauseRepeat1 => "extends_clause_repeat1",
            Type::ImplementsClauseRepeat1 => "implements_clause_repeat1",
            Type::ExtendsTypeClauseRepeat1 => "extends_type_clause_repeat1",
            Type::EnumBodyRepeat1 => "enum_body_repeat1",
            Type::TemplateLiteralTypeRepeat1 => "template_literal_type_repeat1",
            Type::ObjectTypeRepeat1 => "object_type_repeat1",
            Type::TypeParametersRepeat1 => "type_parameters_repeat1",
            Type::TupleTypeRepeat1 => "tuple_type_repeat1",
            Type::ImportSpecifier => "import_specifier",
            Type::NamespaceExport => "namespace_export",
            Type::PropertyIdentifier => "property_identifier",
            Type::ShorthandPropertyIdentifier => "shorthand_property_identifier",
            Type::ShorthandPropertyIdentifierPattern => {
                "shorthand_property_identifier_pattern"
            }
            Type::StatementIdentifier => "statement_identifier",
            Type::ThisType => "this_type",
            Type::TypeIdentifier => "type_identifier",
            Type::Spaces => "Spaces",
            Type::Directory => "Directory",
            Type::ERROR => "ERROR",
        }
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Identifier,
    Type::HashBangLine,
    Type::Export,
    Type::Star,
    Type::Default,
    Type::Type,
    Type::Eq,
    Type::As,
    Type::Namespace,
    Type::LBrace,
    Type::Comma,
    Type::RBrace,
    Type::Typeof,
    Type::Import,
    Type::From,
    Type::Var,
    Type::Let,
    Type::Const,
    Type::Bang,
    Type::Else,
    Type::If,
    Type::Switch,
    Type::For,
    Type::LParen,
    Type::RParen,
    Type::Await,
    Type::In,
    Type::Of,
    Type::While,
    Type::Do,
    Type::Try,
    Type::With,
    Type::Break,
    Type::Continue,
    Type::Debugger,
    Type::Return,
    Type::Throw,
    Type::SemiColon,
    Type::Colon,
    Type::Case,
    Type::Catch,
    Type::Finally,
    Type::Yield,
    Type::LBracket,
    Type::RBracket,
    Type::LT,
    Type::GT,
    Type::Slash,
    Type::Dot,
    Type::Class,
    Type::Async,
    Type::Function,
    Type::TS0,
    Type::TS1,
    Type::New,
    Type::PlusEq,
    Type::DashEq,
    Type::StarEq,
    Type::SlashEq,
    Type::PercentEq,
    Type::CaretEq,
    Type::AmpEq,
    Type::PipeEq,
    Type::GtGtEq,
    Type::GtGtGtEq,
    Type::LtLtEq,
    Type::TS2,
    Type::TS3,
    Type::TS4,
    Type::TS5,
    Type::DotDotDot,
    Type::AmpAmp,
    Type::PipePipe,
    Type::GtGt,
    Type::GtGtGt,
    Type::LtLt,
    Type::Amp,
    Type::Caret,
    Type::Pipe,
    Type::Plus,
    Type::Dash,
    Type::Percent,
    Type::TS6,
    Type::LTEq,
    Type::EqEq,
    Type::TS7,
    Type::BangEq,
    Type::TS8,
    Type::GTEq,
    Type::TS9,
    Type::Instanceof,
    Type::Tilde,
    Type::Void,
    Type::Delete,
    Type::PlusPlus,
    Type::DashDash,
    Type::TS10,
    Type::TS11,
    Type::StringFragment,
    Type::EscapeSequence,
    Type::Comment,
    Type::TS12,
    Type::TS13,
    Type::RegexPattern,
    Type::RegexFlags,
    Type::Number,
    Type::PrivatePropertyIdentifier,
    Type::Target,
    Type::This,
    Type::Super,
    Type::True,
    Type::False,
    Type::Null,
    Type::Undefined,
    Type::At,
    Type::Static,
    Type::Readonly,
    Type::Get,
    Type::Set,
    Type::QMark,
    Type::Declare,
    Type::Public,
    Type::Private,
    Type::Protected,
    Type::Override,
    Type::Module,
    Type::Any,
    Type::Boolean,
    Type::String,
    Type::Symbol,
    Type::Abstract,
    Type::Require,
    Type::Extends,
    Type::Implements,
    Type::Global,
    Type::Interface,
    Type::Enum,
    Type::TS14,
    Type::TS15,
    Type::Asserts,
    Type::Infer,
    Type::Is,
    Type::Keyof,
    Type::Unknown,
    Type::Never,
    Type::Object,
    Type::TS16,
    Type::TS17,
    Type::TS18,
    Type::TS19,
    Type::TS20,
    Type::Program,
    Type::ExportStatement,
    Type::ExportClause,
    Type::ExportSpecifier,
    Type::Declaration,
    Type::ImportStatement,
    Type::ImportClause,
    Type::TS21,
    Type::NamespaceImport,
    Type::NamedImports,
    Type::ExpressionStatement,
    Type::VariableDeclaration,
    Type::LexicalDeclaration,
    Type::VariableDeclarator,
    Type::StatementBlock,
    Type::ElseClause,
    Type::IfStatement,
    Type::SwitchStatement,
    Type::ForStatement,
    Type::ForInStatement,
    Type::TS22,
    Type::WhileStatement,
    Type::DoStatement,
    Type::TryStatement,
    Type::WithStatement,
    Type::BreakStatement,
    Type::ContinueStatement,
    Type::DebuggerStatement,
    Type::ReturnStatement,
    Type::ThrowStatement,
    Type::EmptyStatement,
    Type::LabeledStatement,
    Type::SwitchBody,
    Type::SwitchCase,
    Type::SwitchDefault,
    Type::CatchClause,
    Type::FinallyClause,
    Type::ParenthesizedExpression,
    Type::Expression,
    Type::PrimaryExpression,
    Type::YieldExpression,
    Type::ObjectPattern,
    Type::AssignmentPattern,
    Type::ObjectAssignmentPattern,
    Type::Array,
    Type::ArrayPattern,
    Type::NestedIdentifier,
    Type::ClassDeclaration,
    Type::ClassHeritage,
    Type::FunctionDeclaration,
    Type::GeneratorFunction,
    Type::GeneratorFunctionDeclaration,
    Type::ArrowFunction,
    Type::TS23,
    Type::TS24,
    Type::CallExpression,
    Type::NewExpression,
    Type::AwaitExpression,
    Type::MemberExpression,
    Type::SubscriptExpression,
    Type::AssignmentExpression,
    Type::TS25,
    Type::AugmentedAssignmentExpression,
    Type::TS26,
    Type::TS27,
    Type::SpreadElement,
    Type::TernaryExpression,
    Type::BinaryExpression,
    Type::UnaryExpression,
    Type::UpdateExpression,
    Type::SequenceExpression,
    Type::TemplateString,
    Type::TemplateSubstitution,
    Type::Regex,
    Type::MetaProperty,
    Type::Arguments,
    Type::Decorator,
    Type::ClassBody,
    Type::FormalParameters,
    Type::Pattern,
    Type::RestPattern,
    Type::MethodDefinition,
    Type::Pair,
    Type::PairPattern,
    Type::TS28,
    Type::ComputedPropertyName,
    Type::PublicFieldDefinition,
    Type::NonNullExpression,
    Type::MethodSignature,
    Type::AbstractMethodSignature,
    Type::FunctionSignature,
    Type::TypeAssertion,
    Type::AsExpression,
    Type::ImportRequireClause,
    Type::ExtendsClause,
    Type::ImplementsClause,
    Type::AmbientDeclaration,
    Type::AbstractClassDeclaration,
    Type::InternalModule,
    Type::TS29,
    Type::ImportAlias,
    Type::NestedTypeIdentifier,
    Type::InterfaceDeclaration,
    Type::ExtendsTypeClause,
    Type::EnumDeclaration,
    Type::EnumBody,
    Type::EnumAssignment,
    Type::TypeAliasDeclaration,
    Type::AccessibilityModifier,
    Type::OverrideModifier,
    Type::RequiredParameter,
    Type::OptionalParameter,
    Type::TS30,
    Type::OmittingTypeAnnotation,
    Type::OptingTypeAnnotation,
    Type::TypeAnnotation,
    Type::TS31,
    Type::OptionalType,
    Type::RestType,
    Type::TS32,
    Type::ConstructorType,
    Type::PrimaryType,
    Type::TemplateType,
    Type::TemplateLiteralType,
    Type::InferType,
    Type::ConditionalType,
    Type::GenericType,
    Type::TypePredicate,
    Type::TypePredicateAnnotation,
    Type::TypeQuery,
    Type::IndexTypeQuery,
    Type::LookupType,
    Type::MappedTypeClause,
    Type::LiteralType,
    Type::ExistentialType,
    Type::FlowMaybeType,
    Type::ParenthesizedType,
    Type::PredefinedType,
    Type::TypeArguments,
    Type::ObjectType,
    Type::CallSignature,
    Type::PropertySignature,
    Type::TypeParameters,
    Type::TypeParameter,
    Type::DefaultType,
    Type::Constraint,
    Type::ConstructSignature,
    Type::IndexSignature,
    Type::ArrayType,
    Type::TupleType,
    Type::ReadonlyType,
    Type::UnionType,
    Type::IntersectionType,
    Type::FunctionType,
    Type::ProgramRepeat1,
    Type::ExportStatementRepeat1,
    Type::ExportClauseRepeat1,
    Type::NamedImportsRepeat1,
    Type::VariableDeclarationRepeat1,
    Type::SwitchBodyRepeat1,
    Type::ObjectRepeat1,
    Type::ObjectPatternRepeat1,
    Type::ArrayRepeat1,
    Type::ArrayPatternRepeat1,
    Type::StringRepeat1,
    Type::StringRepeat2,
    Type::TemplateStringRepeat1,
    Type::ClassBodyRepeat1,
    Type::FormalParametersRepeat1,
    Type::ExtendsClauseRepeat1,
    Type::ImplementsClauseRepeat1,
    Type::ExtendsTypeClauseRepeat1,
    Type::EnumBodyRepeat1,
    Type::TemplateLiteralTypeRepeat1,
    Type::ObjectTypeRepeat1,
    Type::TypeParametersRepeat1,
    Type::TupleTypeRepeat1,
    Type::ImportSpecifier,
    Type::NamespaceExport,
    Type::PropertyIdentifier,
    Type::ShorthandPropertyIdentifier,
    Type::ShorthandPropertyIdentifierPattern,
    Type::StatementIdentifier,
    Type::ThisType,
    Type::TypeIdentifier,
    Type::Spaces,
    Type::Directory,
    Type::ERROR,
];