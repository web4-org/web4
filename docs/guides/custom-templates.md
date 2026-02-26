# Custom HTML Templates

The gateway renders W4 pages to HTML using **Tera** — a Jinja2-style template engine for Rust.
You can associate a custom Tera template with any W4 document to control exactly how it looks.

---

## Associating a Template

Add a `<link rel="template">` to the document's `<head>`:

```xml
<head>
  <link rel="template" href="templates/my-theme.html.tera" type="text/x-tera"/>
</head>
```

- `href` is a path relative to the document root (the directory containing the `.w4` file).
- Remote URLs are allowed only if `rendering.template_loader.allow_remote` is `true` in config
  and the host appears in `allowed_remote_hosts`.
- If no template is declared, the built-in default template is used.

---

## Template Context Variables

The gateway injects a `page` object into every template render:

```
page
├── id            string    Canonical document URI
├── title         string    Document title (first <title> element)
├── lang          string    Language tag from the title element
├── meta          map       All <meta name="..."> values keyed by name
├── sections      array     Section objects (see below)
├── services      array     Service objects (see below)
└── metrics       object    Aggregate statistics
    ├── total_sections   int
    ├── total_services   int
    ├── hidden_sections  int   (load="never" sections)
    └── hidden_services  int   (load="never" services)
```

### Section object

```
section
├── id       string
├── name     string
├── class    string
├── load     string   "eager" | "lazy" | "never"
└── content  string   Raw text content of the section
```

### Service object

```
service
├── id       string
├── name     string
├── class    string
├── load     string
├── intent   string   Service intent text
├── consent  string   "open" | "capability" | "interactive"
└── effects  string   "none" | "read" | "write" | "admin"
```

---

## Minimal Template

```html
<!doctype html>
<html lang="{{ page.lang }}">
<head>
  <meta charset="utf-8">
  <title>{{ page.title }}</title>
</head>
<body>
  <h1>{{ page.title }}</h1>

  {% for section in page.sections %}
    {% if section.load != "never" %}
      <section id="{{ section.id }}" class="{{ section.class }}">
        <h2>{{ section.name }}</h2>
        <p>{{ section.content }}</p>
      </section>
    {% endif %}
  {% endfor %}

  <h2>Services</h2>
  {% for svc in page.services %}
    {% if svc.load != "never" %}
      <div class="service {{ svc.class }}">
        <h3>{{ svc.name }}</h3>
        <p>{{ svc.intent }}</p>
        <span>Consent: {{ svc.consent }}</span>
        <span>Effects: {{ svc.effects }}</span>
      </div>
    {% endif %}
  {% endfor %}
</body>
</html>
```

---

## Load Strategy in Templates

The template is responsible for implementing the visual distinction between `eager` and `lazy`
elements. The `never` elements should be excluded from rendering (they are still counted in
`page.metrics`):

```html
{% for section in page.sections %}
  {% if section.load == "never" %}
    {# skip — load=never elements are not rendered #}
  {% elif section.load == "lazy" %}
    <details>
      <summary>{{ section.name }}</summary>
      <p>{{ section.content }}</p>
    </details>
  {% else %}
    <section>
      <h2>{{ section.name }}</h2>
      <p>{{ section.content }}</p>
    </section>
  {% endif %}
{% endfor %}
```

---

## Class-Based Styling

The `class` attribute on sections and services is passed through as a string. Templates can
use it directly in HTML `class` attributes:

```html
<section class="w4-section {{ section.class }}">
```

This allows W4 documents to carry semantic class hints (`hero`, `panel`, `svc-open`) that
templates can style differently.

---

## Metadata

Access `<meta>` values by name:

```html
{% if page.meta.description %}
  <meta name="description" content="{{ page.meta.description }}">
{% endif %}

{% if page.meta["theme-color"] %}
  <meta name="theme-color" content="{{ page.meta['theme-color'] }}">
{% endif %}
```

---

## Template Size Limit

Remote and local templates are subject to the `max_bytes` limit in config
(`rendering.template_loader.max_bytes`, default 256 KiB). Templates exceeding this size
are rejected with an internal error.

---

## Showcase Template

The repository includes a production-quality example template:

```
examples/web4-root-showcase/templates/neon-showcase.html.tera
```

It demonstrates:
- Dark theme with CSS custom properties
- Eager/lazy/never rendering strategy
- Service cards with consent mode badges
- Metric summary display
- JavaScript for interactive service invocation from the browser
