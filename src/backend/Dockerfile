FROM mcr.microsoft.com/dotnet/sdk:7.0-alpine AS publish
WORKDIR /src
COPY backend.csproj ./

RUN dotnet restore backend.csproj --runtime alpine-x64
COPY . .

RUN dotnet publish backend.csproj -c Release -o /app/publish \
    --no-restore \ 
    --runtime alpine-x64 \
    --self-contained true \
    /p:PublishTrimmed=true \
    /p:PublishSingleFile=true

# Building final image used in running container
FROM mcr.microsoft.com/dotnet/runtime-deps:6.0-alpine AS final

RUN adduser --disabled-password \
  --home /app \
  --gecos '' dotnetuser && chown -R dotnetuser /app

USER dotnetuser

WORKDIR /app
EXPOSE 80
COPY --from=publish /app/publish .
ENV ASPNETCORE_URLS=http://*:8080

ENTRYPOINT ["./backend"]
