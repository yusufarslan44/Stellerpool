# Stellerpool — korumalı ev/araç tasarruf havuzu

> Birlikte biriktir. Her şeyi doğrula. — *Save together. Verify everything.*

Stellerpool, FuzulEv/FuzulOto tarzı düzenli birikim ve sıralı teslimat fikrinin Stellar / Soroban üzerinde bir prototipidir. Hedef, grubun parasını organizatörün serbestçe kullanabildiği bir hesapta toplamak yerine kurallı bir akıllı kontratta tutmak ve ev/araç tahsisatını doğrulanmış satıcıya yönlendirmektir.

Rise In x Stellar **Pro Hackathon 2026** (19–20 Eylül, İstanbul) için geliştiriliyor. Track: Genesis. Ayrıntılı ürün ve güvenlik tasarımı: [plan](docs/plan.md).

## Durum

Proje geliştirme aşamasında. Ön yüz çalışması sürüyor; Soroban kontratı, sponsor güvencesi, satıcı doğrulaması ve gerçek TL anchor akışı henüz tamamlanmadı. Aşağıdaki kurallar **hedef tasarımdır**, bugün çalışan ürün özelliği olarak sunulmaz.

## Sorun ve yaklaşım

Ortak tasarruf grubunda iki risk var: organizatörün toplanan fonu amacı dışında kullanması ve erken tahsisat alan üyenin sonraki katkıları ödememesi. Kontrat ilk risk için fon hareketini sınırlar. İkinci risk ekonomik güvence gerektirir: eski plandaki bir taksitlik üye teminatı yeterli değildir.

- Havuz başlamadan ayrı bir **sponsor güvencesi** kontrata yatırılır. Dört üye ve tur başına 10 birimlik örnekte gerekli başlangıç güvencesi 40 birimdir.
- Katkılar, sponsor güvencesi ve her üyenin iade hakkı ayrı muhasebeleştirilir. Organizatörün serbest çekim yetkisi yoktur.
- Üyeler ve tahsisat sırası başladıktan sonra kilitlenir.
- Tur katkıları tam ve satıcı/alım onayı hazırsa, kontrat gerekli iade bakiyesi kaldığını kontrol edip tutarı doğrudan satıcı cüzdanına yollar.
- Katkı eksikse tur durur. Sponsor açığı ayrıca karşılamazsa güvenli iptal ve iade akışı işletilir. Tahsisat almamış üyelerin yatırdığı katkıların iadesi önceliklidir.
- TL giriş veya çıkışı için anchor gerekir; hangi sağlayıcının kullanılacağı henüz netleşmedi.

Bu tasarım gerçek mülkiyet devrini, gelecekteki taksit tahsilatını veya TL token'ının değerini tek başına garanti etmez. Gerçek para ve ev/araç işlemleri için lisanslı ortak, hukuki inceleme, kimlik/satıcı doğrulaması ve bağımsız kontrat denetimi gerekir.

## Örnek akış

~~~mermaid
flowchart TD
  A[Havuz ve sıra belirlenir] --> B[Sponsor güvencesi kontrata kilitlenir]
  B --> C[Üyeler katılır ve tur katkısını öder]
  C --> D{Üye katkıları veya sponsor tamamlamasıyla tur tutarı hazır mı?}
  D -->|Evet| E[Satıcı ve alım kaydı doğrulanır]
  E --> F{Ödeme sonrası iade hakkı karşılanıyor mu?}
  F -->|Evet| G[Tahsisat satıcıya ödenir]
  G --> H[Sonraki tur]
  D -->|Süre doldu| I[Tur durur]
  F -->|Hayır| I
  I --> K{Sponsor açığı tamamlar mı?}
  K -->|Evet| D
  K -->|Hayır| J[Havuz iptal edilir, hak sahiplerine iade]
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

- [ ] Sponsor güvencesi, tahsisat ve iade değişmezleri test edilmiş Soroban kontratı
- [ ] Testnet contract ID ve yeniden üretilebilir kurulum
- [ ] Cüzdanlı arayüz ve satıcıya demo ödeme
- [ ] Eksik ödeme → durdurma → iptal/iade demosu
- [ ] Kabul edilen TL anchor giriş veya çıkış akışı
- [ ] Gerçek ve simüle parçaları ayıran demo/sunum dokümanı
