{{- define "trieve.redisUrl" -}}
{{- if .Values.redis.enabled -}}
{{- $svcName := printf "%s-redis-master" .Release.Name -}}
{{- $redisSecretName := printf "%s-redis" .Release.Name -}}
{{- $redisSecretKey := "redis-password" -}}
{{- $redisUri := printf "redis://%s@%s" .Release.Name -}}
{{- with (lookup "v1" "Secret" .Release.Namespace $redisSecretName) -}}
{{- $redisPassword := index .data $redisSecretKey | b64dec -}}
redis://{{ $redisPassword }}@{{ $svcName }}
{{- end -}}
{{- else -}}
{{ .Values.config.redis.uri }}
{{- end -}}
{{- end -}}

{{- define "trieve.databaseUrl" -}}
{{/* DATABASE_URL */}}
{{ if $.Values.postgres.dbURI }}
DATABASE_URL: {{ .Values.postgres.dbURI }}
{{- else if $.Values.postgres.secretKeyRef }}
{{- $secretName := .Values.postgres.secretKeyRef.name }}
{{- $secretKey := .Values.postgres.secretKeyRef.key }}
{{- with (lookup "v1" "Secret" .Release.Namespace $secretName) }}
DATABASE_URL: {{ index .data $secretKey | b64dec }}
{{- end }}
{{- else }}
{{- $secretName := printf "%s-trieve-postgres-app" .Release.Name -}}
{{- with (lookup "v1" "Secret" .Release.Namespace $secretName) }}
DATABASE_URL: {{ index .data "uri" | b64dec }}
{{- end }}
{{- end }}
{{/* End DATABASE_URL */}}
{{- end }}
