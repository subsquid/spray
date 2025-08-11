use jsonrpsee::core::BoxError;
use jsonrpsee::server::{HttpRequest, HttpResponse};
use prometheus_client::registry::Registry;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};


pub struct MetricsLayer {
    registry: Arc<Registry>
}


impl MetricsLayer {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self {
            registry
        }
    }
}


impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware {
            inner,
            registry: self.registry.clone()
        }
    }
}



pub struct MetricsMiddleware<S> {
    inner: S,
    registry: Arc<Registry>
}


impl<S> Service<HttpRequest> for MetricsMiddleware<S>
where
    S: Service<HttpRequest, Response = HttpResponse>,
    S::Response: 'static,
    S::Error: Into<BoxError> + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        if req.uri().path() == "/metrics" {
            let registry = self.registry.clone();

            return Box::pin(async move {
                let mut metrics = String::new();

                prometheus_client::encoding::text::encode(&mut metrics, &registry)
                    .expect("String IO is infallible");

                let res = HttpResponse::builder()
                    .status(200)
                    .header("content-type", "text/plain; charset=utf-8")
                    .body(metrics.into())
                    .expect("response is valid");

                Ok(res)
            })
        }

        let fut = self.inner.call(req);

        Box::pin(async move {
            fut.await.map_err(Into::into)
        })
    }
}


impl <S: Clone> Clone for MetricsMiddleware<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            registry: self.registry.clone()
        }
    }
}
