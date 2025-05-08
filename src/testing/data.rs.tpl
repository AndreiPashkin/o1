// This file is auto-generated. Do not edit directly.
// Generated using gomplate from data.rs.tpl

//! Test data for static hash map implementations

#![allow(clippy::large_const_arrays)]

{{ $data := datasource "data" }}

{{ range $typeName, $typeInfo := $data }}
// {{ $typeInfo.type }} array ({{ len $typeInfo.data }} elements)
pub const {{ $typeName }}: [({{ $typeInfo.type }}, u64); {{ len $typeInfo.data }}] = [
{{- if eq $typeInfo.type "&str" }}
{{- range $key, $value := $typeInfo.data }}
    ("{{ $key }}", {{ $value }}),
{{- end }}
{{- else if eq $typeInfo.type "bool" }}
{{- range $key, $value := $typeInfo.data }}
    ({{ $key }}, {{ $value }}),
{{- end }}
{{- else }}
{{- range $key, $value := $typeInfo.data }}
    ({{ $key }}, {{ $value }}),
{{- end }}
{{- end }}
];
{{ end }}
