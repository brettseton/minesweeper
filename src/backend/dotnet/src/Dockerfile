FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:8.0-alpine AS publish
ARG TARGETARCH
WORKDIR /src
COPY backend.csproj ./

# Map Docker TARGETARCH to .NET RID and restore
RUN arch=$TARGETARCH \
    && if [ "$arch" = "amd64" ]; then rid="linux-musl-x64"; \
    elif [ "$arch" = "arm64" ]; then rid="linux-musl-arm64"; \
    else rid="linux-musl-$arch"; fi \
    && echo "Restoring for RID: $rid" \
    && dotnet restore backend.csproj -r $rid

COPY . .

# Publish using the calculated RID
RUN arch=$TARGETARCH \
    && if [ "$arch" = "amd64" ]; then rid="linux-musl-x64"; \
    elif [ "$arch" = "arm64" ]; then rid="linux-musl-arm64"; \
    else rid="linux-musl-$arch"; fi \
    && echo "Publishing for RID: $rid" \
    && dotnet publish backend.csproj -c Release -o /app/publish \
    -r $rid \
    --no-restore \
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
EXPOSE 8080
COPY --from=publish /app/publish .
ENV ASPNETCORE_URLS=http://*:8080

ENTRYPOINT ["./backend"]