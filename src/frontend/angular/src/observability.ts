import { WebTracerProvider } from '@opentelemetry/sdk-trace-web';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-web';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { ZoneContextManager } from '@opentelemetry/context-zone';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { XMLHttpRequestInstrumentation } from '@opentelemetry/instrumentation-xml-http-request';
import { FetchInstrumentation } from '@opentelemetry/instrumentation-fetch';
import { UserInteractionInstrumentation } from '@opentelemetry/instrumentation-user-interaction';
import { DocumentLoadInstrumentation } from '@opentelemetry/instrumentation-document-load';
import { Resource } from '@opentelemetry/resources';
import { MeterProvider, PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-http';
import { LoggerProvider, BatchLogRecordProcessor } from '@opentelemetry/sdk-logs';
import { OTLPLogExporter } from '@opentelemetry/exporter-logs-otlp-http';
import { logs } from '@opentelemetry/api-logs';
import { metrics } from '@opentelemetry/api';

export function initObservability() {
  const resource = new Resource({
    'service.name': 'angular-frontend',
    'deployment.environment': 'development',
  });

  // 1. Tracing
  const tracerProvider = new WebTracerProvider({ resource });
  tracerProvider.addSpanProcessor(new BatchSpanProcessor(new OTLPTraceExporter({
    url: '/v1/traces',
  })));
  tracerProvider.register({
    contextManager: new ZoneContextManager(),
  });

  // 2. Metrics
  const meterProvider = new MeterProvider({ resource });
  meterProvider.addMetricReader(new PeriodicExportingMetricReader({
    exporter: new OTLPMetricExporter({ url: '/v1/metrics' }),
    exportIntervalMillis: 60000,
  }));
  metrics.setGlobalMeterProvider(meterProvider);

  // 3. Logs
  const loggerProvider = new LoggerProvider({ resource });
  loggerProvider.addLogRecordProcessor(new BatchLogRecordProcessor(new OTLPLogExporter({
    url: '/v1/logs',
  })));
  logs.setGlobalLoggerProvider(loggerProvider);

  // 4. Instrumentations
  registerInstrumentations({
    instrumentations: [
      new DocumentLoadInstrumentation(),
      new UserInteractionInstrumentation(),
      new XMLHttpRequestInstrumentation({
        propagateTraceHeaderCorsUrls: [/.*/],
      }),
      new FetchInstrumentation({
        propagateTraceHeaderCorsUrls: [/.*/],
      }),
    ],
  });

  console.log('OpenTelemetry initialized');
}