use web4_core::W4mlParser;

const MINIMAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/index.w4">
  <head>
    <title lang="en">Web4 Demo</title>
    <meta name="description" content="A minimal Web4 page."/>
    <link rel="canonical" href="https://example.com/index.w4"/>
  </head>
  <body>
    <section id="intro" name="Introduction">
      <p>Hello</p>
      <!-- comment should be ignored -->
      <ext:note xmlns:ext="urn:ext">trace text</ext:note>
    </section>
    <schema>
      <type id="AddInput" kind="object">
        <property name="a" type="int" required="true"/>
      </type>
      <type id="AddOutput" kind="object">
        <property name="sum" type="int" required="true"/>
      </type>
    </schema>
    <service id="math.add" kind="tool" sourceRef="services/math.add.w4s">
      <intent lang="en">Add numbers</intent>
      <input typeRef="AddInput"/>
      <output typeRef="AddOutput"/>
      <effects level="none"/>
      <consent mode="open"/>
    </service>
  </body>
</w4>"#;

#[test]
fn parses_annex_style_document_and_builds_index() {
    let parser = W4mlParser;
    let parsed = parser.parse(MINIMAL).expect("parse should succeed");
    assert!(parsed.model.is_some());
    let index = parsed.index.expect("index available");
    assert_eq!(index.section_ids.get("intro"), Some(&0));
    assert_eq!(index.service_ids.get("math.add"), Some(&0));
    assert_eq!(index.type_ids.get("AddInput"), Some(&0));
}

#[test]
fn rejects_duplicate_schema() {
    let parser = W4mlParser;
    let source =
        r#"<w4 xmlns="urn:w4ml:0.1" version="0.1"><head/><body><schema/><schema/></body></w4>"#;
    assert!(parser.parse(source).is_err());
}

#[test]
fn rejects_unresolved_typeref() {
    let parser = W4mlParser;
    let source = r#"<w4 xmlns="urn:w4ml:0.1" version="0.1"><head/><body><service id="s.test" sourceRef="services/s.test.w4s"><input typeRef="Missing"/></service></body></w4>"#;
    assert!(parser.parse(source).is_err());
}

#[test]
fn rejects_inline_bindings() {
    let parser = W4mlParser;
    let source = r#"<w4 xmlns="urn:w4ml:0.1" version="0.1"><head/><body><service id="s.test" sourceRef="services/s.test.w4s"><bindings><binding type="http" method="POST" endpoint="/rpc/s.test"/></bindings></service></body></w4>"#;
    assert!(parser.parse(source).is_err());
}

#[test]
fn self_closing_section_is_accepted() {
    let parser = W4mlParser;
    let source =
        r#"<w4 xmlns="urn:w4ml:0.1" version="0.1"><head/><body><section id="intro" /></body></w4>"#;
    assert!(parser.parse(source).is_ok());
}

#[test]
fn parses_annex_b1_minimal_page() {
    let parser = W4mlParser;
    let source = r#"<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1"
    id="https://example.com/index.w4">

  <head>
    <title lang="en">Web4 Demo</title>
    <meta name="description" content="A minimal Web4 page."/>
    <link rel="canonical" href="https://example.com/index.w4"/>
    <link rel="template"  href="https://example.com/templates/default.html"
          type="text/html"/>
  </head>

  <body>

    <section id="intro" name="Introduction" load="eager">
      <p>This is a Web4 page. It carries both content and callable services.</p>
      <ul>
        <li>Structured knowledge for agents</li>
        <li>Callable tools in the same document</li>
        <li>Discoverable via standard HTTP</li>
      </ul>
    </section>

    <schema>
      <type id="AddInput" kind="object">
        <property name="a" type="int" required="true" desc="First operand."/>
        <property name="b" type="int" required="true" desc="Second operand."/>
        <additionalProperties value="false"/>
      </type>
      <type id="AddOutput" kind="object">
        <property name="sum" type="int" required="true"/>
      </type>
    </schema>

    <service id="math.add" name="add" kind="tool" load="eager"
             sourceRef="services/math.add.w4s">
      <intent lang="en">Add two integers and return their sum.</intent>
      <input  typeRef="AddInput"/>
      <output typeRef="AddOutput"/>
      <effects level="none"/>
      <consent mode="open"/>
      <policy>
        <rateLimit value="60/m"/>
        <allowOrigins value="*"/>
        <costHint latencyMs="5" price="0"/>
      </policy>
      <errors>
        <error code="INVALID_ARGUMENT" retryable="false"/>
        <error code="RATE_LIMITED"     retryable="true"/>
      </errors>
      <examples>
        <example>
          <call>{"a": 2, "b": 3}</call>
          <result>{"sum": 5}</result>
        </example>
      </examples>
    </service>

  </body>
</w4>"#;
    assert!(parser.parse(source).is_ok());
}

#[test]
fn parses_annex_b2_service_when_embedded() {
    let parser = W4mlParser;
    let source = r#"<w4 xmlns="urn:w4ml:0.1" version="0.1" id="https://example.com/index.w4">
  <head><title>demo</title></head>
  <body>
    <schema>
      <type id="ChatInput" kind="object"><property name="thread_id" type="string" required="false"/></type>
      <type id="ChatOutput" kind="object"><property name="answer" type="string" required="true"/></type>
    </schema>
    <service id="agent.chat" name="chat" kind="agent" load="eager"
             sourceRef="services/agent.chat.w4s">
      <intent lang="en">Engage in structured multi-turn dialogue with this agent.</intent>
      <input  typeRef="ChatInput"/>
      <output typeRef="ChatOutput"/>
      <effects level="read"/>
      <consent mode="capability">
        <issue   endpoint="/consent/issue" method="POST"/>
        <present header="Authorization"    scheme="W4-Capability"/>
      </consent>
      <policy>
        <rateLimit value="20/m"/>
        <allowAgents value="*"/>
      </policy>
    </service>
  </body>
</w4>"#;
    assert!(parser.parse(source).is_ok());
}
