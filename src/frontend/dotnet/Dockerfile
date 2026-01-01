FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:8.0-alpine AS publish
ARG TARGETARCH
WORKDIR /src
COPY . .

RUN arch=$TARGETARCH \
    && if [ "$arch" = "amd64" ]; then rid="linux-musl-x64"; \
    elif [ "$arch" = "arm64" ]; then rid="linux-musl-arm64"; \
    else rid="linux-musl-$arch"; fi \
    && echo "Building for RID: $rid" \
    && dotnet publish frontend.csproj -c Release -o /app/publish \
    -r $rid \
    --self-contained true \
    /p:PublishTrimmed=false \
    /p:PublishSingleFile=true

# Building final image used in running container
FROM mcr.microsoft.com/dotnet/runtime-deps:8.0-alpine AS final

RUN adduser --disabled-password \
  --home /app \
  --gecos '' dotnetuser && chown -R dotnetuser /app

USER dotnetuser

WORKDIR /app
EXPOSE 80
COPY --from=publish /app/publish .
ENV ASPNETCORE_URLS=http://*:8080

ENTRYPOINT ["./frontend"]
