# Stellerpool — korumalı ev/araç tasarruf havuzu planı

**Amaç:** FuzulEv/FuzulOto benzeri düzenli birikim ve sıralı teslimat deneyimini Stellar üzerinde, fonları organizatörün kullanımından ayırarak kurmak. Ana güven vaadi: organizatör ortak parayı kendi hesabına çekemez; her tahsisatın koşulları, alıcısı ve tutarı denetlenebilir.

**Aşama ve karar (19 Eylül 2026):** Şimdi lisans başvurusu veya gerçek müşteri fonu yok. Ürün, davetli altın günü fikrini anlatan bir **Mainnet tanıtım sitesi** ile fon akışının yalnızca test varlığıyla sınanacağı ayrı bir **Testnet demosu** olarak ilerler. Anchor, gerçek TRY dönüşümü yerine açıkça simüle edilen adım akışı olarak kalır. Mainnet'teki mevcut site salt okunurdur; kontrat, cüzdan imzası veya ödeme içermez. Anchor akış simülasyonu ekranda çalışır; havuz kontratı ve AI denetçisi henüz tamamlanmadı. [Dağıtım ve kapsam kaydı](mainnet-showcase.md) bu ayrımı açıklar.

**Hackathon ölçütü:** DOC gerçek TL giriş/çıkışı ile kullanılabilir Stellar bakiyesi istiyor. Seçilen simülasyon bu şartı **karşılamaz**; sunumda eksik kalem olarak belirtilecek. [AI denetimi ve Türkiye'de gerçek ürün yolu](legal-ai-path.md) ileride değerlendirilebilecek ayrı akışları anlatır.

**Hukuki kapsam:** Bu plan, Fuzul benzeri ev/araç finansmanı vaadini inceler. Kapalı arkadaş grubu şeklindeki altın günüyle aynı hukuki statüde olduğu varsayılmamalıdır; [farklar ve açık sorular](altin-gunu-legal-boundary.md) ayrıca yazıldı.

**Hackathon:** Rise In x Stellar Pro Hackathon 2026, Genesis; teslim 20 Eylül 2026 12:00.

## 1. Referans model ve farkımız

Fuzul'ün resmi anlatımında kişiler ödeme güçlerine göre gruplara ayrılıyor; çekilişli ve teslim tarihi baştan belirlenen seçenekler bulunuyor. Teslimat sonrasında taksitler sürüyor. Fuzul, teslimat öncesinde taksit dondurmanın teslimatı da ertelediğini, grup dağılmasına karşı organizasyonun arkasında durduğunu ve teslim edilen ev/araç için ipotek veya rehin uyguladığını söylüyor. BDDK, ödemesini aksatan tasarruf dönemi müşterisinin tahsisatının sözleşmeye göre ertelenebileceğini veya sözleşmesinin feshedilebileceğini açıklıyor. Fuzul'ün kamuya açık SSS'si teslimat **sonrası** tahsilatın tüm adımlarını açıklamıyor. Bizim prototipimiz bu kurumsal, hukuki ve bilanço güvencelerinin yerine geçmez.

| İhtiyaç | Prototipteki karşılık | Açık sınır |
|---|---|---|
| Organizatör parayı alıp kaçamasın | Katkı ve sponsor güvencesi Soroban kontratında; creator'ın serbest çekim yetkisi yok | Kontrat hatası, anahtar ve varlık ihraççısı riski sürer |
| Sıra ve ödemeler değiştirilemesin | Grup başladıktan sonra üye, katkı, sıra ve süre kilitli; olaylar zincirde | Kura ve esnek ödeme planları MVP dışında |
| Erken teslim alan taksitleri bırakırsa diğerleri korunabilsin | Havuz başlamadan sponsorun ayrı varlığı kilitlenir; güvenli devam mümkün değilse bekleyenlere iade | Sponsor ekonomik kaybı üstlenir; gerçek dünyadaki tahsilat ayrı iştir |
| Tahsisat amaç dışına gitmesin | Alıcı, satıcı adresini ve alım kaydını önerir; doğrulama sonrası ödeme doğrudan satıcı cüzdanına | Tapu, ruhsat ve gerçek satıcı kimliği zincir dışında doğrulanır |
| Şüpheli sözleşme veya alım belgesi fark edilsin | Planlanan AI denetçisi sürüm, tutar ve satıcı tutarsızlığını gerekçeli raporlar; insanlar inceler | AI raporu tek başına hukuki onay, kontrat denetimi veya ödeme yetkisi değildir |
| TL bağlantısı anlaşılsın | Demo ekranında TRY → test varlığı adımları simüle edilebilir; gerçek aktarım ve kullanılabilir TRY bakiyesi yoktur | Simülasyon hackathon'un gerçek TL şartını karşılamaz; gerçek anchor ve izinler ayrıca değerlendirilir |

## 2. Ürün kararı

1. **MVP: sabit sıralı, sponsor güvenceli havuz.** Üye başına bir teslimat ve her dönem bir katkı. Kura ve bireysel teslim tarihi varyantları sonraya bırakılır.
2. **Organizatör yalnızca kurulum önerir.** Sıra, doğrulayıcılar, ödeme takvimi ve diğer koşullar tüm üyeler ile sponsor tarafından cüzdanlarıyla onaylanmadan havuz başlamaz. Her değişiklik önceki onayları geçersiz kılar. Organizatör başladıktan sonra sırayı veya satıcıyı tek başına değiştiremez, kontrat fonunu çekemez. Kontratta tek taraflı upgrade/kaçış kapısı bulunmaz.
3. **Katılımcıdan girişte bir taksitlik teminat alınmaz.** Eski plan bu teminatı yeterli koruma gibi sunuyordu; dört üyeli örnekte iki taksitlik açık bırakıyordu. Ayrıca tasarruf dönemindeki teminat kuralları hukuken incelenmelidir.
4. **Sponsor güvencesi yalnızca Testnet senaryosudur.** Demo sponsoru ileride Testnet varlığını yatırır. Bu mekanizma Mainnet tanıtımında veya gerçek altın günü hizmetinde finansman garantisi diye sunulmaz. Gerçek üründe sponsorun kim olacağı, fonun niteliği ve düzenleyici statü hukuk ve iş ortaklığı kararıdır.
5. **Gecikme teslimat öncesi ve sonrası ayrı ele alınır.** Önce bildirim ve sözleşmede belirlenmiş ek süre gelir. Teslimat almamış üyenin kendi tahsisat hakkı ertelenebilir. Teslimat almış üyenin borcu sürer; sponsor açığı ayrıca yatırıp bekleyenleri koruyabilir. Güvenli devam sağlanamazsa ek süre sonunda iptal ve iade devreye girer. Başka üyelerin alacağı sessizce azaltılmaz.
6. **Tahsisat satıcıya yapılır.** Demo için önceden belirlenmiş test satıcısı cüzdanı kullanılır ve gerçek ev/araç satın alındığı söylenmez. Sıradaki üye kendi tur katkısını bizzat ödemiş ve eski sponsor avanslarını kapatmış olmalıdır; sponsor onun yerine ödeme yaparak tahsisatı açamaz. Gerçek ürün, satıcı ve mülkiyet doğrulama ortağı gerektirir.
7. **Getiri stratejisi yok.** Ortak fonu lending, staking veya likidite havuzuna yatırmak MVP güven vaadini geniş risklere açar. DeFi özelliği programlanabilir saklama ve doğrulanabilir kurallardır.
8. **AI denetçisi yardımcıdır.** Demo belge ve koşullarını karşılaştırıp insan doğrulayıcıya gerekçeli bulgu sunar; Soroban kodundaki olası yetki/muhasebe hatalarını geliştiriciye işaretler. Fon tutmaz, kontratta özel anahtarı veya bağımsız ödeme/onay yetkisi bulunmaz. AI kod incelemesi bağımsız güvenlik denetimi değildir. Belirsiz AI çıktıları müşterinin iade hakkını veya yasal süreleri değiştiremez. Üretim sözleşmesinin tarafı, kimlik ve imza süreci, fon ve tahsisat yetkisi [hukuki yolda](legal-ai-path.md) tanımlandığı gibi lisanslı yapıyla kurulmalıdır.

### Ödeme aksadığında izlenecek yol

| Aşama | Fuzul/BDDK kaynaklarında doğrulanabilen | Stellerpool için karar |
|---|---|---|
| Teslimat öncesi | Fuzul taksit dondurmanın teslimatı ertelediğini söylüyor; BDDK sözleşmeye göre tahsisatın ötelenebileceğini veya sözleşmenin feshedilebileceğini açıklıyor. | Gecikme bildirimi ve ek süre; ödeme yapılmazsa ilgili üyenin tahsisatı durur. Küçük ve sabit sıralı MVP grubunda başka üyelerin sırası tek taraflı değiştirilmez. Üye değiştirme/yeni sıra gerçek ürünün ayrı tasarımıdır. |
| Teslimat sonrası | Fuzul teslim edilen ev/araç üzerinde ipotek veya rehin bulunduğunu söylüyor; kamuya açık SSS kesin bir ihtar, yapılandırma veya tahsilat takvimi vermiyor. | Borç kaydı ve bildirim; sözleşme ve lisanslı ortak varsa yapılandırma/tahsilat. Mülkiyet devri, ödeme ve gerekli ipotek/rehin işlemleri yetkili ortakla eşleştirilir. Sponsor, tahsilatı beklerken diğer üyelerin iade hakkını karşılar. |
| İyileşme olmazsa | Gerçek dünyadaki alacak ve teminat işlemleri sözleşme ve hukuk yoluyla yürütülür. | Testnet'te ek süre sonunda sponsor katkı yapmazsa iptal/iade. Gerçek üründe alacağın takibi ile iptal/iade birbirinden ayrı yürür; kontrat mülke kendiliğinden el koyamaz. |

Bildirim, ek süre ve yeniden yapılandırma sırası **bizim ürün önerimizdir**; Fuzul'ün açıklamadığı iç tahsilat süreci olarak sunulmaz. Teslimat sonrası üyeye süre tanınması, sponsor finansmanı veya başka bir onaylı kaynak olmadan diğer üyelerin alacağını eksiltemez. Sponsorun eksik katkıyı yatırması borcu silmez; gerçek para sürümünde kime borç doğacağı sözleşmeyle belirlenir. Sabit sıralı MVP'de sıradaki üye ek sürede kendi borcunu kapatmazsa onun yerine sponsor ödemesiyle tahsisat yapılmaz; havuz iptal/iade sürecine gider. Sıra atlama veya borçlu üyeyi değiştirme ayrı, herkesin önceden kabul ettiği bir ürün tasarımı gerektirir.

## 3. Ekonomik güvence hesabı

N üye, kişi başına her tur C katkı, N tur ve her tur bir tahsisat varsayılır. r tamamlanmış turdan sonra henüz tahsisat almamış N-r kişinin o ana kadar yatırdığı toplam tutar r × (N-r) × C'dir. Bu kişilerin havuz durursa yatırdıklarını geri alabilmesi için kontrat bakiyesi en az bu tutarı karşılamalıdır. Başlangıçta kilitlenecek sponsor güvencesinin alt sınırı, bu dizinin tepe noktası olan **floor(N²/4) × C**'dir. Bu formül yalnızca bu sabit, eşit katkılı model ve aşağıdaki iade kuralı içindir; gerçek varlık değeri, kur ve hukuki tahsilat riskini kapsamaz.

**Örnek:** Dört üye, her tur 10 birim öder. Sponsor başta 40 birim kilitler. İlk tur sonunda A'nın satıcısına 40 ödenir; B, C ve D'nin önceki toplam 30 birimi için kontratta 40 kalır. İkinci turda herkes öderse B'nin satıcısına 40 ödenir ve henüz teslim almamış C ile D'nin yatırdığı 40 birim kontratta kalır. İlk teslim alan A ikinci turda ödemezse tahsisat kendiliğinden gerçekleşmez: sponsor tamamlayıp iade yeterliliğini koruyabilir veya havuz iptal edilir ve bekleyenler yatırdıklarını kontrattan geri alır. Sponsor bu durumda kayıp yaşayabilir.

**İade hakkı:** İptalde teslimat almamış üyeler bizzat yaptıkları tüm katkıları; teslimat almış üyeler yalnızca henüz tamamlanmamış tur için bizzat yatırdıkları katkıyı geri alır. Tamamlanmış turların katkısı geri alınamaz. Sponsorun üye yerine yaptığı tamamlama üyenin ödemesi veya iade alacağı sayılmaz. Sponsor, tüm üye iadelerinden sonra kalan havuz bakiyesini alır; dışarıdaki alacağını kontrat bu bakiyenin ötesinde garanti etmez. Aynı varlık için havuzlar arası borç/alacak mahsuplaşması yapılmaz.

**Sponsor avansı ayrı kayıttır:** Bir turun eksik katkısını sponsor yeni fon yatırarak tamamladığında ilgili üye için avans/borç kaydı oluşur. Üye avansı geri öderse ödeme sponsora gider ve borç kaydı kapanır; geçmiş turun üye katkısı veya iade hakkı geriye dönük artmaz. Başlangıç güvencesi, tur tamamlaması ve üye katkısı üç ayrı muhasebe kalemidir. `floor(N²/4) × C` başlangıç güvencesi, yalnızca yukarıdaki iade yükümlülüğünün teorik alt sınırıdır; her tahsisat öncesi fiili bakiye kontrolü zorunludur. N büyüdükçe güvence ikinci dereceden artar: 100 üyede 2500 × C, yani bir turun toplam katkısının 25 katı gerekir. Gerçek üründe sponsor sermayesi, zarar fiyatlaması ve grup üst sınırı ayrıca kurulmalıdır; demo için en çok 12 üye kabul edilir.

## 4. Kullanıcı akışı

~~~mermaid
flowchart TD
  A[Havuz ve bitiş tarihi oluştur] --> B[Sponsor güvencesini kontrata kilitle]
  B --> C[Üyeler katılır; sıra ve koşulları herkes onaylar]
  C --> D{Kuruluş süresi doldu mu?}
  D -->|Evet, başlamadı| O[İptal ve hak sahiplerine iade]
  D -->|Hayır, koşullar tamam| S[Herkes havuzu başlatabilir]
  S --> E[Tur katkıları yatırılır]
  E --> F{Üye katkıları veya sponsor tamamlamasıyla tur tutarı hazır mı?}
  F -->|Evet| R{Sıradaki üye kendi katkısını ve eski avanslarını kapattı mı?}
  R -->|Evet| G[Alıcı satıcıyı ve alım kaydını önerir]
  G --> AI[AI varsa gerekçeli risk raporu sunar]
  AI --> H[Bağımsız insan doğrulaması ve imzalar; satıcı varlık alabilir]
  H --> T{Alım için son süre doldu mu?}
  T -->|Evet| O
  T -->|Hayır| I{Ödeme sonrası iade yeterliliği korunuyor mu?}
  I -->|Evet| J[Tutar doğrudan satıcıya gider]
  J --> K{Son tur mu?}
  K -->|Hayır| E
  K -->|Evet| L[Sponsor kalanı geri alır]
  F -->|Süre doldu| M[Tur durur, üye bilgilendirilir]
  R -->|Hayır, ek süre doldu| O
  M --> P[Ek süre ve teslimat öncesi/sonrası durumu]
  P --> N{Üye öder veya sponsor açık tutarı tamamlar mı?}
  N -->|Evet: ödeme kayda geçer| F
  N -->|Hayır| O
  I -->|Hayır| O
~~~

Demo sırasında alım doğrulaması imzalı test verisidir; gerçek satıcı veya tapu entegrasyonu değildir. AI adımı planlanmıştır, çalışan entegrasyon olarak sunulmaz. Model erişilemezse insan doğrulaması ve kontratın kesin kuralları geçerlidir; AI raporu sözleşmeyi resmîleştirmez. Kontrat fon transferini doğrular, fiziksel mülkiyet devrini kendi başına bilemez. Zaman aşımı zincirde otomatik işlem başlatmaz; süre dolunca herkes iptal/iade çağrısını yapabilir.

## 5. Kontrat tasarımı

**Durumlar ve saatler:** Havuz `Filling → Active → Completed / Aborted`; her tur `Collecting → AwaitingPurchase → Settled` geçişlerini kullanır, ödeme veya eski avans kapanışı gecikirse `Collecting → Grace → AwaitingPurchase` olur. Katkı süresi `start_pool` veya önceki `execute_round` ile başlar. Tur hazır değilken katkı son tarihi geçince herkes turu `Grace` yapabilir; ek süre **ilk katkı son tarihinden** hesaplanır, geç `mark_overdue` çağrısıyla uzamaz. Ek süre sonunda eksik ödeme veya sıradaki üyenin açık avansı varsa herkes havuzu iptal edebilir. Tüm katkılar tamamlanıp sıradaki üye kendi katkısını ve eski avanslarını kapattığı işlemde alım için ayrı, sınırlı süre başlar. Satıcı önerisini değiştirmek bu süreyi sıfırlamaz. Bu sürede geçerli satıcı önerisi ve yeterli onay gelmezse herkes iptali tetikleyebilir. Ödeme öncesi bakiye kontrolü başarısızsa tahsisat yapılmaz; iade yalnızca fiili bakiye tüm hakları karşılıyorsa tam olarak ödenebilir. Başlama, katkı, ek süre ve alım son tarihleri oluşturulan koşullarda sabittir; kontrat çağrısı olmadan zincirde kendiliğinden işlem olmaz.

**P0 fonksiyonlar**
- create_pool: varlık, katkı, en çok 12 üye, tur/ek süre/alım süreleri, kuruluş son tarihi, sponsor ve demo için izinli test satıcısını belirler. Demo satıcısı creator veya sponsor adresi olamaz. Ekonomik koşullar değiştirilemez; farklı tutar veya süre için yeni havuz gerekir. Tarihler sıralı, süreler pozitiftir.
- fund_guarantee: sponsor güvencesini ayrı muhasebe kaydına yatırır; yeterli güvence olmadan havuz başlamaz. Sponsor fonu kuruluş süresince kilitlidir.
- join_pool: üyeyi kaydeder; cüzdan ve varlık uygunluğunu doğrular. Demo satıcısıyla aynı üye adresi kabul edilmez; farklı adresler aynı gerçek kişiye ait olabilir.
- propose_terms, approve_terms: creator sırayı ve doğrulayıcı adreslerini önerir; tüm üyeler ve sponsor, ekonomik koşulları da içeren aynı sürümü onaylar. Sıra veya doğrulayıcılar değişirse bütün onaylar silinir. Doğrulayıcı adresleri creator, sponsor, üyeler ve birbirinden farklıdır; demo için en az 2/3 eşik kullanılır. Adres ayrılığı gerçek kişi/kurum bağımsızlığını kanıtlamaz.
- start_pool: tüm koltuklar dolu, aynı koşul sürümü herkesçe onaylı ve güvence tam ise **herkes** havuzu başlatabilir; creator çevrim dışı kalsa da fon kilitli kalmaz.
- cancel_unstarted_pool: kuruluş son tarihi geçip havuz başlamadıysa herkes iptal edebilir; sponsor kilitlediği tutarın tamamını alır. Bu aşamada üyeden tur katkısı alınmaz.
- deposit, cure_payment: üyenin mevcut tur katkısını bir kez alır; ek sürede ödeme aynı borcu kapatır. Sponsor tarafından karşılanmış katkı üyenin kendi katkısı sayılmaz.
- top_up, repay_advance: sponsor eksik katkıyı **yeni fonla** ilgili üye ve tur adına tamamlar; başlangıç güvencesi buraya aktarılmaz. Cari turda sıradaki üye adına top_up kabul edilmez. Üyenin sponsora geri ödemesi eski avansı kapatır, fakat geçmiş turu ikinci kez finanse etmez veya yeni iade hakkı oluşturmaz. Sıradaki üye kendi katkısı ve eski avansları açıkken tahsisat alamaz.
- propose_purchase: yalnızca sıradaki üye, alım süresi içinde satıcı adresi, varlık/tutar ve zincir dışı belge özetini sürümlü kaydeder. Demo satıcısı havuzun izinli test satıcısı olmalıdır. Değişiklik önceki tüm alım onaylarını siler.
- approve_purchase: onay, havuz + tur + alıcı + satıcı + varlık + tutar + belge özeti + öneri sürümüne bağlanır; en az 2/3 doğrulayıcı gerekir. Demo satıcısı önceden belirlenmiş ayrı cüzdandır; gerçek satıcı kimliği ve danışıklı işlem riski için yetkili dış doğrulama gerekir.
- AI risk incelemesi (zincir dışı, planlanan): mevcut koşul sürümü ve demo alım belgesi için gerekçeli uyarı üretir. İnsan doğrulayıcı AI çıktısını görebilir; AI adresi doğrulayıcı eşiğine dahil edilmez. Gerekirse rapor sürümünün özeti kayda bağlanır; modelin serbest metni `execute_round` önkoşulu değildir. Gerçek müşteri verileri bu akışa girmez.
- execute_round: tüm katkılar üye veya kayıtlı sponsor avansıyla tam, sıradaki üye **kendi** cari katkısını ödemiş ve eski avanslarını kapatmış, onaylar geçerli, satıcı varlığı alabilir ve ödeme sonrası tüm iade hakları karşılanabilir ise yalnızca o turun tutarını kayıtlı satıcıya yollar. Herkes çağırabilir; herhangi bir önkoşul yoksa transfer yapılmaz.
- mark_overdue, abort_pool: turun hazır olmaması için ek süreyi işaretler. Ek sürenin veya alım süresinin sonunda koşullar sağlanmadıysa herkes iptali tetikleyebilir. Bakiye değişmezi başarısızsa fon çıkışı durur ve durum güvenlik olayı olarak görünür; eksik bakiyeyle tam iade vaadi yapılamaz. Bekleme yalnızca sözleşmede belirli sürelerle sınırlıdır.
- claim_refund, claim_sponsor_remainder: iptal veya tamamlanma durumuna göre hak sahibine iade. İki kez talep engellenir.
- get_pool, get_round, get_member_status, get_refund_claim, get_sponsor_advance: okuma.

**Değişmezler**
- Hiçbir fonksiyon havuzun toplam kontrat bakiyesini doğrudan tahsisat tutarı saymaz.
- Her transfer belirli pool_id, tur ve hak sahibi ile ilişkilidir; başka havuzun varlığı kullanılamaz.
- Her tahsisat sonrasında kontratın o havuza atanan bakiyesi, teslimat almamış üyelerin tüm katkıları ile teslimat almış üyelerin henüz tamamlanmamış tur katkılarından doğan iade hakları toplamından az olamaz; aynı katkı iki kez sayılmaz.
- Üye katkısı, sponsor avansı ve başlangıç güvencesi ayrı kayıtlardır. Sponsor avansı üyenin iade hesabına katılmaz; aynı tur ödemesi iki kaynakla iki kez tahsil edilmez.
- Sıradaki üyenin cari katkısı kendi cüzdanından gelmiş ve eski sponsor avansları kapatılmış olmalıdır; aksi halde o tur satıcısına ödeme yoktur.
- Onaylar yalnızca geçerli koşul/alım sürümü için sayılır. Satıcı değişince önceki imzalar kullanılamaz. Satıcı varlığın transferini kabul edebilmelidir; Stellar klasik hesapta gerekli trustline/izin yoksa ödeme durur.
- Sponsor güvencesi, üye alacakları kapatılmadan çekilemez.
- Aynı üye aynı turda ikinci kez ödeme veya aynı iade için ikinci kez talep yapamaz.
- Kuruluş, başlama, katkı, ek süre ve alım son tarihleri zincirde saklanır; fon veya iade hakkı dururken depolama TTL'sinin yenilenmesi ve arşivden geri getirme akışı tasarlanıp test edilir. İptal ve iade süre dolunca herkesçe tetiklenebilir.
- Varlık tutarları tamsayı en küçük birimle saklanır; arayüzde string/BigInt kullanılır.

**Güvenlik:** Yetkilendirme, taşma, yeniden giriş/çapraz kontrat çağrısı, çok havuzlu muhasebe, kötü niyetli satıcı, hatalı doğrulama, eksik ödeme, iade yarışı ve TTL test edilir. Özel durum testleri: grup dolmazken sponsor iadesi; creator yokken başlatma; koşul/satıcı değişince onayların sıfırlanması; doğrulayıcı adres çakışması; sponsorca tamamlanmış sıradaki üyenin ödeme alamaması; açık avans; satıcı veya onay yokken alım süresi aşımı; satıcının varlığı alamaması; sponsor avansının üye iadesine eklenmemesi; havuzlar arası bakiye sızıntısı. Gerçek fonlardan önce bağımsız kontrat denetimi gerekir.

## 6. Zincir dışı parçalar ve mevzuat

- **Anchor demosu:** Seçilen kapsamda gerçek banka/TRY yatırma, çekme veya token ihraç entegrasyonu yapılmaz. Mevcut arayüz, para tutarı veya kişisel bilgi almadan giriş/çıkış adımlarını yerel simülasyon olarak gösterir; ödeme talimatı, cüzdan imzası ve zincir işlemi üretmez. Her adım “Simülasyon” diye etiketlenir ve kullanılabilir bakiye oluşturulmadığı açıkça söylenir. Demo, hackathon DOC'unun gerçek TL şartını karşılamış sayılmaz. Gerçek sağlayıcıya bağlanma ayrı hukuki ve teknik karar gerektirir.
- **Varlık:** Testnet havuz senaryosu yalnızca test varlığı kullanır. Gelecekte gerçek anchor düşünülürse varlık, ihraççı, geri ödeme hakkı, freeze/clawback yetkileri ve kur riski ayrıca gösterilir. TL olmayan varlıkta taksit ile TL satın alma gücü aynı şey değildir.
- **Gerçek ev/araç:** Satıcı kimliği, fatura/sözleşme, tapu/ruhsat ve ipotek/rehin işlemlerinin ödeme ile güvenli biçimde eşleşmesi zincir dışı doğrulayıcı veya lisanslı ortak gerektirir. Doğrulayıcı yanlış bilgi verirse kontratın doğru ödeme yapması tek başına kaybı önlemez. Kontrat, ihtar veya taşınmaz/araç üzerindeki hukuki tahsilatı yürütemez.
- **Hukuki model:** [Ayrıntılı hukuki yol](legal-ai-path.md): lisanslı şirket müşteriyle usulüne uygun elektronik sözleşme kurar; ayrılmış TL fon havuzu ve satıcıya banka ödemesi uygulanır. Cüzdan imzası veya AI incelemesi bu sözleşmeye resmiyet ya da faaliyet izni vermez. Dış hizmet, müşteri hakları, tahsisat zamanları, sponsor güvencesi, ipotek/rehin, KYC/AML ve ödeme hizmetleri uzmanla ve yetkili kurumlarla netleştirilir. Testnet demosu faaliyet izni anlamına gelmez. Katılımcıdan tasarruf döneminde nakit teminat alma fikri ayrıca değerlendirilmeden geri getirilmeyecek.
- **Çekiliş:** MVP'de yok. Gelecekte eklenirse doğrulanabilir rastgelelik, katılım ve iptal kuralları ayrıca tasarlanacak.

## 7. Hackathon demosu ve teslim ölçütü

1. Dört üye ve bir demo sponsoru Testnet cüzdanlarıyla katılır; örnek katkı 10 birim, sponsor güvencesi 40 birim. Üyeler ve sponsor aynı sıra/koşul sürümünü onaylar; herkes başlatabilir.
2. Arayüz, havuz bakiyesini, sponsor güvencesini, bekleyen üyelerin iade hakkını, sırayı ve tur son tarihini ayrı gösterir.
3. TRY → test varlığı anchor yolculuğu hazırlanırsa tüm tutarlar ve adımlar simülasyon olarak işaretlenir; gerçekte banka, anchor ve cüzdan bakiyesi değişmez. Hackathon DOC'unun gerçek TL ve kullanılabilir bakiye ölçütü karşılanmadığı açıkça belirtilir.
4. İlk turda herkes öder; önceden belirlenmiş ve varlığı alabilen demo satıcısına 40 birim gider. Zincir işlemi ve satıcı adresi gösterilir.
5. İkinci turda daha önce tahsisat alan A ödemez; ödeme süresi dolunca tur durur, gecikme ve ek süre gösterilir. A ödemez ve sponsor tamamlamazsa ek süre sonunda iptal tetiklenir; teslimat almamış üyeler katkılarını geri alır ve sponsorun zararı görünür.
6. Pozitif ikinci senaryoda A ek sürede bizzat öder **veya** sponsor A'nın açığını yeni fonla kapatır; B kendi katkısını ödemişse ve iade yeterliliği korunuyorsa B'nin turu tamamlanır. Sponsorun A'ya yaptığı avans A'nın iade hakkı sayılmaz. Ayrı negatif senaryoda sıradaki alıcı B kendi katkısını ödemez; sponsor B adına top_up yapamaz ve B'nin satıcısına ödeme açılmaz.
7. Testler: normal tur, farklı havuzların ayrılığı, dolmayan grupta sponsor iadesi, creator yokken başlama, sıra/koşul onayı, alım onayı sıfırlama, sıradaki borçlu üyenin bloklanması, eksik trustline/izin, alım süresi aşımı, avans/üye iadesi ayrımı, yetersiz güvence, çift talep ve muhasebe değişmezi.
8. Testnet contract ID, kurulum adımları, demo URL, gerçek/simüle edilen parçaların sınırları ve sunum hazırlanır.
9. AI denetçisi yetişirse tutarsız demo alımını gerekçeli olarak işaretleyen gerçek model çalışması ve insan doğrulayıcı kararı gösterilir; yetişmezse özellik planlanan olarak sunulur. AI bir satıcıya ödemeyi kendi başına açamaz ve kontrat fonksiyonu yerine geçmez.

**Arayüz eşleştirmesi (P0):** Kontrat ABI'si hazır olunca frontend durumları havuz ve tur olarak ayrılacak; `Grace` ve `AwaitingPurchase`, `cure_payment`, kuruluş iptali, koşul/onay sürümü, alım son tarihi, alım önerisi yenileme, sponsor avansı ile üyenin kendi katkısı ve sıradaki üyenin ödeme engeli arayüzde gösterilecek. Mevcut arayüzde bu alanların bulunması kontrat kuralı yerine geçmez; kontrat ve istemci aynı durum geçişlerini kullanana kadar demo tamamlanmış sayılmaz.

**Öncelik:** Mainnet tanıtımında işlemleri kapalı ve kapsamı açık tut; Testnet için önce fon muhasebesi ve iptal/iade, ardından demo satıcı ödemesi ve cüzdan uyumunu tamamla. Anchor simülasyonu ayrı ve gerçek bakiye üretmeyen bir anlatım olarak kalır. AI denetçisi yalnızca çalıştırılıp kanıtlandığında demo özelliği sayılır. Gerçek para havuzu veya ev/araç teslim vaadi yapılmaz.

## 8. Açık kararlar

1. Sponsor kim olacak ve gerçek üründe kaybı hangi bilanço taşıyacak?
2. Satıcı ve mülkiyet doğrulamasını hangi yetkili taraf yapacak? Sahte satıcı ve danışıklı işlem nasıl engellenecek?
3. Seçilen anchor gerçek TL'yi hangi ağda, hangi varlık ve minimum tutarla destekliyor?
4. Düzenlenmiş tasarruf finansmanı kapsamındaki gerçek ürün için hangi lisanslı şirket sözleşme tarafı olacak? AI ve teknik sağlayıcının görevi Yönetmelik md. 32 ile nasıl uyumlu tutulacak?
5. Havuz başına ayrı kontrat mı kullanılacak? Tek kontrat, ayrı muhasebeye rağmen ortak kod riski taşır.
6. Üyelerin para birimi TL ise token/TL kur değişimi ve satın alma gücü riski kime ait?
7. Elektronik sözleşme, uzaktan kimlik, müşteri hakları ve belge arşivi hangi yetkili kurumun sistemlerinde yürütülecek? AI incelemesi ve insan kararı nasıl kayıt altına alınacak?

## 9. Araştırma kaynakları

- Fuzul sistemin işleyişi: https://www.fuzulev.com.tr/nasil-calisir
- Fuzul SSS (teslimat sonrası taksit, grup güvencesi, ipotek/rehin): https://www.fuzulev.com.tr/merak-edilenler
- BDDK tasarruf finansman SSS (gecikme, tahsisat erteleme, fon ayrımı ve müşteri hakları): https://www.bddk.org.tr/Sss/Liste/117
- BDDK lisanslı şirket listesi: https://www.bddk.org.tr/Kurulus/Liste/89
- 6361 sayılı Kanun, 39/A ve 39/B: https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=6361&MevzuatTur=1&MevzuatTertip=5
- Tasarruf Finansman Yönetmeliği, 17, 21-23 ve 32: https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=38500&MevzuatTur=7&MevzuatTertip=5
- BDDK uzaktan kimlik ve elektronik sözleşme yönetmeliği, 12-13: https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=39368&MevzuatTur=7&MevzuatTertip=5
- Stellar Asset Contract ve ihraççı yetkileri: https://developers.stellar.org/docs/tokens/stellar-asset-contract
- Stellar varlık trustline doğrulama: https://developers.stellar.org/docs/build/guides/basics/verify-trustlines

Kaynaklar 19 Eylül 2026 tarihinde incelendi. Bu belge ürün ve teknik planıdır; hukuki görüş veya gerçek para güvencesi değildir.