{{- define "trieve.worker.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .name }}
  labels:
    app.kubernetes.io/name: {{ .name }}
    app.kubernetes.io/instance: {{ $.instance }}
  annotations:
    checksum/config: {{ include (print $.Template.BasePath "/settings/backend-configmap.yaml") . | sha256sum }}
spec:
  replicas: {{ .replicas | default 1 }}
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ .name }}
      app.kubernetes.io/instance: {{ $.Release.Name }}
  template:
    metadata:
      labels:
        app.kubernetes.io/name: {{ .name }}
        app.kubernetes.io/instance: {{ $.Release.Name }}
    spec:
      {{- if $.Values.global.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml $.Values.global.imagePullSecrets | nindent 8 }}
      {{- end }}
      containers:
      - name: {{ .name }}
        image: {{ printf "%s/%s:%s" $.Values.global.image.registry .image.repository .image.tag }}
        imagePullPolicy: {{ $.Values.global.imagePullPolicy }}
        envFrom:
          - configMapRef:
              name: trieve-server-config
        {{- if .env }}
        env:
          {{- range .env }}
          - {{ . | toYaml | nindent 12 }}
          {{- end }}
        {{- end }}
        {{- if .resources }}
        resources:
          {{- toYaml .resources | nindent 12 }}
        {{- end }}
{{- end -}}
