# Stellerpool — korumalı ev/araç tasarruf havuzu

> Birlikte biriktir. Her şeyi doğrula. — *Save together. Verify everything.*

Stellerpool, FuzulEv/FuzulOto tarzı düzenli birikim ve sıralı teslimat fikrinin Stellar / Soroban üzerinde bir prototipidir. Hedef, grubun parasını organizatörün serbestçe kullanabildiği bir hesapta toplamak yerine kurallı bir akıllı kontratta tutmak ve ev/araç tahsisatını doğrulanmış satıcıya yönlendirmektir.

Rise In x Stellar **Pro Hackathon 2026** (19–20 Eylül, İstanbul) için geliştiriliyor. Track: Genesis. Ayrıntılı ürün ve güvenlik tasarımı: [plan](docs/plan.md).

## Durum

Proje geliştirme aşamasında. Ön yüz çalışması sürüyor; Soroban kontratı, sponsor güvencesi, satıcı doğrulaması ve gerçek TL anchor akışı henüz tamamlanmadı. Aşağıdaki kurallar **hedef tasarımdır**, bugün çalışan ürün özelliği olarak sunulmaz.

## Sorun ve yaklaşım

Ortak tasarruf grubunda iki risk var: organizatörün toplanan fonu amacı dışında kullanması ve erken tahsisat alan üyenin sonraki katkıları ödememesi. Kontrat ilk risk için fon hareketini sınırlar. İkinci risk ekonomik güvence gerektirir: eski plandaki bir taksitlik üye teminatı yeterli değildir.

- Havuz başlamadan ayrı bir **sponsor güvencesi** kontrata yatırılır. Dört üye ve tur başına 10 birimlik örnekte gerekli başlangıç güvencesi 40 birimdir.
- Üye katkıları, sponsorun başlangıç güvencesi, sponsor avansları ve her üyenin iade hakkı ayrı muhasebeleştirilir. Organizatörün serbest çekim yetkisi yoktur.
- Üyeler ve sponsor aynı sıra/koşul sürümünü onaylamadan havuz başlamaz. Kuruluş süresi dolarsa sponsor fonunu geri alabilir; koşullar tamamsa herkes havuzu başlatabilir.
- Tur katkıları tam, sıradaki üyenin kendi katkısı ve eski avansları kapalı, satıcı/alım onayı geçerli ve satıcı varlığı alabilir durumda ise kontrat iade bakiyesi kaldığını kontrol edip tutarı doğrudan satıcı cüzdanına yollar.
- Katkı eksikse ek süre başlar. Sıradaki üye adına sponsor katkısı tahsisatı açamaz. Başka üyenin açığını sponsor yeni fonla kapatabilir; bu avans o üyenin iade hakkı sayılmaz. Ek süre sonunda koşullar sağlanmazsa iptal ve iade tetiklenebilir.
- Alım için ayrıca son tarih vardır. Satıcı veya doğrulayıcı onayı gelmezse fonlar süresiz kilitlenmez; iptal ve iade yolu açılır.
- TL giriş veya çıkışı için anchor gerekir; hangi sağlayıcının kullanılacağı henüz netleşmedi.

Bu tasarım gerçek mülkiyet devrini, gelecekteki taksit tahsilatını veya TL token'ının değerini tek başına garanti etmez. Gerçek para ve ev/araç işlemleri için lisanslı ortak, hukuki inceleme, kimlik/satıcı doğrulaması ve bağımsız kontrat denetimi gerekir.

Fuzul'ün kamuya açık açıklamalarında teslimat öncesi taksit dondurma ve teslimat erteleme ile teslimat sonrası ev/araç üzerinde ipotek veya rehin bulunuyor. Gerçek üründe mülkiyet devri, ödeme, hukuki güvence ve borç takibi kontrat dışındaki yetkili ortakla eşleştirilmelidir. Sponsor güvencesi, tahsilat sürerken bekleyen üyelerin parasını korumak içindir; borçlunun borcunu otomatik silmez. Ayrıntılar ve kaynaklar [planda](docs/plan.md).

## Örnek akış

~~~mermaid
flowchart TD
  A[Havuz kurulur, sponsor fonu kilitlenir] --> B[Üyeler ve sponsor sıra ile koşulları onaylar]
  B --> C{Kuruluş süresinde hazır mı?}
  C -->|Hayır| J[İptal ve iade]
  C -->|Evet| D[Herkes havuzu başlatabilir]
  D --> E[Tur katkıları toplanır]
  E --> F{Katkılar tam ve sıradaki üye borçsuz mu?}
  F -->|Evet| G[Satıcı ve alım onayı için sınırlı süre]
  G --> H{Onay, satıcı ve iade bakiyesi uygun mu?}
  H -->|Evet| I[Tahsisat satıcıya ödenir]
  I --> E
  F -->|Ek süre sonunda hayır| J
  H -->|Alım süresi sonunda hayır| J
~~~

## Teknoloji ve klasörler

| Klasör | Amaç |
|---|---|
| contracts/ | Rust + Soroban kontratı ve testleri |
| frontend/ | Vue 3 + TypeScript, Stellar Wallets Kit arayüzü |
| backend/ | Gerekirse anchor ve gerçek alım doğrulaması için ince servis |
| scripts/ | Testnet deploy ve demo yardımcıları |
| docs/ | [Ürün planı](docs/plan.md), Stellar notları ve mimari |

Stellar Testnet, Stellar SDK, RPC ve uygun bir Stellar varlığı kullanılacak. Fonun kontratta olması, varlık ihraççısı ve anchor riskini ortadan kaldırmaz. Prototipte ortak fon DeFi getiri ürünlerine yatırılmaz.

## Teslim durumu

- [x] Sponsor güvencesi, tahsisat ve iade değişmezleri test edilmiş Soroban kontratı (29 birim testi, [docs/IMPLEMENTATION_LOG.md](docs/IMPLEMENTATION_LOG.md), Faz 1-7)
- [ ] Kuruluş ve alım zaman aşımı, koşul onayları, sponsor avansı ve borçlu alıcı engeli testleri — **kontrat henüz bu güncellenmiş plana göre revize edilmedi**, bkz. not aşağıda
- [x] Testnet contract ID ve yeniden üretilebilir kurulum ([scripts/deploy_testnet.sh](scripts/deploy_testnet.sh); canlı kontrat `CACZQBHHQ3TY33AJIC52MHMCPEKUYO4KQFURHQGPJJ3LNDJZAT6LG4II`)
- [ ] Kontrat durumlarıyla uyumlu cüzdanlı arayüz ve satıcıya demo ödeme (satıcıya ödeme kontrat seviyesinde Testnet'te doğrulandı; cüzdan arayüzü üzerinden tıklanabilir akış henüz yok)
- [x] Eksik ödeme → durdurma → iptal/iade demosu ([scripts/demo_testnet.sh](scripts/demo_testnet.sh), kontrat seviyesinde canlı Testnet çalıştırması, [docs/IMPLEMENTATION_LOG.md](docs/IMPLEMENTATION_LOG.md) Faz 10 — bu demo, planın önceki sürümündeki genel top-up/grace/abort kurallarını izliyor; aşağıdaki nota bakın)
- [ ] Kabul edilen TL anchor giriş veya çıkış akışı
- [ ] Gerçek ve simüle parçaları ayıran demo/sunum dokümanı

> **Not (19 Eylül):** `docs/plan.md` bu kontrat teslim edildikten sonra önemli ölçüde revize edildi (kuruluş zaman aşımı/`cancel_unstarted_pool`, tüm tarafların onayladığı `propose_terms`/`approve_terms`, kişi-bazlı sponsor avansı ve sıradaki üyenin kendi borcunu kapatmadan tahsisat alamaması, alım için ayrı son tarih, önceden kayıtlı demo satıcısı). Yukarıda işaretli kontrat maddeleri planın **önceki** sürümünü karşılıyor; yeni kurallar henüz kontrata yansıtılmadı. Bkz. `docs/IMPLEMENTATION_PLAN.md` içindeki "Documentation Conflicts / Open Questions".
