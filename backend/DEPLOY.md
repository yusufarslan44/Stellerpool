# Sunucuya kalıcı deploy (Yusuf'un sunucusu)

Şu an bu servis yalnızca bir Cloudflare quick tunnel (`cloudflared tunnel --url`) ile
geçici olarak dışarı açık — geliştiricinin laptobu kapanınca ölür, adres her yeniden
başlatmada değişir. Bu belge, aynı servisi **kalıcı bir sunucuda, sabit bir alan adıyla**
ayağa kaldırmak için gereken adımları listeler.

## ⚠️ En kritik kural: issuer/distribution anahtarlarını YENİDEN ÜRETME

`TRYT` varlığı ve içindeki arz zaten Testnet'te var (`npm run setup-issuer` bu oturumda
çalıştırıldı, gerçek işlemlerle uçtan uca test edildi).
Sunucuda **tekrar `npm run setup-issuer`
çalıştırmayın** — bu, farklı bir issuer'la tamamen YENİ bir varlık üretir ve şimdiye
kadarki testler/dokümandaki adresler geçersiz kalır.

Bunun yerine: geliştiricinin yerel `backend/.env` dosyasındaki `ISSUER_SECRET` ve
`DISTRIBUTION_SECRET` değerlerini **güvenli bir kanaldan** (şifreli DM, bir parola
yöneticisi paylaşımı — **git'e, Slack/Discord genel kanalına, e-postaya asla**) alıp
sunucudaki `.env`'e aynen yapıştırın.

Mevcut varlık: kod `TRYT`, issuer `GAOPTL4Q34VQWE5PWWVYEX7QQLOSO2DVYUVURCHHXTZFKASS66YL7YJ3`
(bu public key, secret değil — paylaşılması sorun değil, doğrulama için kullanılabilir).

## Gereksinimler

- Node.js 20+ (backend `package.json`'daki `"type": "module"` + `@stellar/stellar-sdk`
  için yeterli; sunucuda `node --version` ile kontrol edin).
- Bu servise ayrılmış bir **alt alan adı**, örn. `anchor.stellerpool.<sizin-domaininiz>`.
  SEP-1 gereği `stellar.toml`'un **tam olarak** `https://<alan-adı>/.well-known/stellar.toml`
  adresinde durması gerekir — bu yüzden ayrı bir path değil, ayrı bir (alt) alan adı şart.
- TLS için bir reverse proxy (Caddy önerilir — otomatik Let's Encrypt sertifikası alır,
  tek satır config yeterli; nginx + certbot de olur).
- Süreç yöneticisi: `pm2` (en hızlı kurulum) veya `systemd` unit dosyası.

## Adımlar

### 1. Kod

```bash
git clone https://github.com/yusufarslan44/Stellerpool.git
cd Stellerpool/backend
npm install
npm run build   # dist/ üretir
```

### 2. `.env`

```bash
cp .env.example .env
```

`.env`'i düzenleyin:

```bash
PORT=3001
PUBLIC_BASE_URL=https://anchor.stellerpool.<sizin-domaininiz>
STELLAR_NETWORK=testnet
HORIZON_URL=https://horizon-testnet.stellar.org
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Yukarıdaki uyarıya göre GÜVENLİ KANALDAN alınan, MEVCUT değerler (yeniden üretmeyin):
ISSUER_SECRET=<paylaşılan değer>
DISTRIBUTION_SECRET=<paylaşılan değer>
ASSET_CODE=TRYT

# Sunucuya özel, yeni ve rastgele üretilebilir (bu sunucunun kendi oturum imzalama anahtarı):
JWT_SECRET=<openssl rand -hex 32 ile üretin>

AUTO_CONFIRM_SECONDS=0
```

`.env` dosyasını **asla commit etmeyin** (`backend/.gitignore` zaten hariç tutuyor).

### 3. DNS

`anchor.stellerpool.<domain>` için bir A/AAAA (veya CNAME) kaydını sunucunun IP'sine
yönlendirin.

### 4. Reverse proxy + TLS (Caddy örneği)

```caddyfile
anchor.stellerpool.<domain> {
    reverse_proxy 127.0.0.1:3001
}
```

`caddy run` (veya sistem servisini kurduysanız `systemctl enable --now caddy`) — Caddy
sertifikayı otomatik alır/yeniler. nginx + certbot kullanıyorsanız eşdeğer bir
`proxy_pass http://127.0.0.1:3001;` bloğu yeterli.

Uygulamanın kendisi yalnızca `127.0.0.1:3001`'i dinlemeli — port 3001'i doğrudan dışarıya
açmayın, TLS ve genel erişim reverse proxy üzerinden olmalı.

### 5. Süreç yöneticisi (pm2 örneği)

```bash
npm install -g pm2
cd Stellerpool/backend
pm2 start dist/server.js --name stellerpool-anchor
pm2 save
pm2 startup   # verdiği komutu çalıştırın; sunucu yeniden başlayınca otomatik ayağa kalkar
```

`systemd` tercih edilirse: `ExecStart=/usr/bin/node /path/to/Stellerpool/backend/dist/server.js`,
`WorkingDirectory=/path/to/Stellerpool/backend`, `EnvironmentFile=/path/to/Stellerpool/backend/.env`,
`Restart=always`.

### 6. Doğrulama

```bash
curl https://anchor.stellerpool.<domain>/.well-known/stellar.toml
curl https://anchor.stellerpool.<domain>/health
```

`WEB_AUTH_ENDPOINT` ve `TRANSFER_SERVER_SEP0024`'ün **yeni alan adını** gösterdiğini
doğrulayın (localhost veya eski tünel adresi değil).

Ardından tam uçtan uca doğrulama:

```bash
node scripts/test-flow.mjs https://anchor.stellerpool.<domain>
```

Çıktının sonunda gerçek bir Stellar işlem hash'i ve `250.0000000 TRYT` bakiyesiyle
"✅ Uçtan uca SEP-1/10/24 akışı başarılı." görmelisiniz — geliştiricinin yerel makinesinde
ve tünel üzerinden iki kez doğrulanmış aynı script.

### 7. Frontend'i buraya bağlamak

`frontend/.env`:

```bash
VITE_ANCHOR_HOME_DOMAIN=anchor.stellerpool.<domain>
```

Ardından frontend'i anchor'a bağlayın (`VITE_POOL_ASSET_CODE=TRYT` / `VITE_POOL_ASSET_ISSUER`).

## Sonradan güncelleme

Kod değiştiğinde: `git pull && npm install && npm run build && pm2 restart stellerpool-anchor`
(veya `systemctl restart stellerpool-anchor`). `.env`'e dokunmayın — issuer/distribution
sırları ve varlık aynı kalmalı.
