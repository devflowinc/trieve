{{/*
Expand the name of the chart.
*/}}
{{- define "qdrant.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "qdrant.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "qdrant.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "qdrant.labels" -}}
helm.sh/chart: {{ include "qdrant.chart" . }}
{{ include "qdrant.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "qdrant.selectorLabels" -}}
app: {{ include "qdrant.name" . }}
app.kubernetes.io/name: {{ include "qdrant.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "qdrant.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "qdrant.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create secret
*/}}
{{- define "qdrant.secret" -}}
{{- $readOnlyApiKey := false }}
{{- $apiKey := false }}
{{- if eq (.Values.apiKey | toJson) "true" -}}
{{- /* retrieve existing randomly generated api key or create new one */ -}}
{{- $secretObj := (lookup "v1" "Secret" .Release.Namespace (printf "%s-apikey" (include "qdrant.fullname" . ))) | default dict -}}
{{- $secretData := (get $secretObj "data") | default dict -}}
{{- $apiKey = (get $secretData "api-key" | b64dec) | default (randAlphaNum 32) -}}
{{- else if .Values.apiKey -}}
{{- $apiKey = .Values.apiKey -}}
{{- end -}}
{{- if eq (.Values.readOnlyApiKey | toJson) "true" -}}
{{- /* retrieve existing randomly generated api key or create new one */ -}}
{{- $secretObj := (lookup "v1" "Secret" .Release.Namespace (printf "%s-apikey" (include "qdrant.fullname" . ))) | default dict -}}
{{- $secretData := (get $secretObj "data") | default dict -}}
{{- $readOnlyApiKey = (get $secretData "read-only-api-key" | b64dec) | default (randAlphaNum 32) -}}
{{- else if .Values.readOnlyApiKey -}}
{{- $readOnlyApiKey = .Values.readOnlyApiKey -}}
{{- end -}}
{{- if and $apiKey $readOnlyApiKey -}}
api-key: {{ $apiKey | b64enc }}
read-only-api-key: {{ $readOnlyApiKey | b64enc }}
local.yaml: {{ printf "service:\n  api_key: %s\n  read_only_api_key: %s" $apiKey $readOnlyApiKey | b64enc }}
{{- else if $apiKey -}}
api-key: {{ $apiKey | b64enc }}
local.yaml: {{ printf "service:\n  api_key: %s" $apiKey | b64enc }}
{{- else if $readOnlyApiKey -}}
read-only-api-key: {{ $readOnlyApiKey | b64enc }}
local.yaml: {{ printf "service:\n  read_only_api_key: %s" $readOnlyApiKey | b64enc }}
{{- end -}}
{{- end -}}