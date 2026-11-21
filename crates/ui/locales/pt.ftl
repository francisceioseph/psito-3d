app-title = Psito 3D

## Barra de ferramentas
surprise-me = Surpreenda-me
new-scene = Nova cena
open-scene = Abrir cena
save-scene = Salvar cena
export-image = Salvar imagem
set-wallpaper = Usar como papel de parede
undo = Desfazer
redo = Refazer
filter-none = Limpa
filter-dither = 256 cores
theme = Tema
language = Idioma
filter-crt = Tela de tubo

## Código da cena
scene-code = Código da cena
scene-code-share = Copie este código para compartilhar este mundo: quem abrir recebe a mesma cena.
scene-code-picture = Este terreno é uma imagem deste computador; outras pessoas vão precisar da mesma imagem.
scene-code-paste = Cole um código para abrir o mundo dele
scene-code-hint = Um código começa com p3d1.
copy-code = Copiar código
code-copied = Copiado
open-code = Abrir
close = Fechar
code-invalid = Este não é um código de cena do Psito 3D, ou falta um pedaço dele.
code-too-new = Este código foi feito por um Psito 3D mais novo.
orbit = Girar sozinho

## Céu
section-sky = Céu
sun = Sol
sun-height = Altura do sol
sun-direction = Direção do sol
haze = Bruma
clouds = Nuvens
cloud-height = Altura das nuvens
exposure = Brilho

## Terreno
section-terrain = Terreno
generator = Forma
generator-missing = Esta forma vem de um plugin que não está instalado.
source-picture = De uma imagem
choose-picture = Escolher imagem
seed = Semente
roll-seed = Sortear outra semente
terrain-height = Escala vertical
coast = Litoral
resolution = Detalhe

## Brushes
section-brushes = Pincéis
brush-off = Desligado
brush-raise = Levantar
brush-lower = Cavar
brush-smooth = Alisar
brush-flatten = Aplainar
brush-size = Tamanho do pincel
brush-strength = Força
brush-hint = Arraste na imagem para pintar o chão. O botão direito gira, o do meio desliza.
clear-strokes = Apagar traços
brush-strokes = { $count ->
    [0] Nenhum traço
    [one] Um traço
   *[other] { $count } traços
}

## Mar e névoa
section-sea = Mar
sea = Mar
sea-color = Cor da água
sea-level = Nível da água
ripple = Ondulação
section-fog = Névoa
fog = Névoa
mist = Neblina nos vales
mist-height = Altura da neblina

## Objetos
section-objects = Objetos
add-sphere = Adicionar uma esfera
add-cube = Adicionar uma caixa
add-cylinder = Adicionar um cilindro
add-cone = Adicionar um cone
add-torus = Adicionar um anel
shape-sphere = Esfera
shape-cube = Caixa
shape-cylinder = Cilindro
shape-cone = Cone
shape-torus = Anel
no-objects = Nenhum objeto ainda. Adicione um com os botões acima.
show-object = Mostrar
hide-object = Esconder
object-name = Nome
material = Material
material-chrome = Cromado
material-gold = Ouro
material-glass = Vidro
material-clay = Argila
color = Cor
roughness = Aspereza
metallic = Metal
transparency = Transparência
combine = Junta-se aos outros
combine-union = Somando
combine-subtract = Cavando
combine-intersect = Cruzando
blend = Derreter
position-x = Esquerda e direita
position-y = Para cima e para baixo
position-z = Perto e longe
size = Tamanho
turn = Girar
tilt = Inclinar
delete-object = Apagar objeto

## Estado
status-samples = Refinando: { $done } de { $total }
status-done = Pronta
terrain-working = Construindo o terreno
terrain-failed = O terreno falhou: { $reason }
plugin-errors = { $count ->
    [one] Um plugin não carregou
   *[other] { $count } plugins não carregaram
}
wallpaper-working = Fazendo o papel de parede
wallpaper-set = Papel de parede trocado
wallpaper-unsupported = este ambiente não tem um jeito conhecido de trocá-lo
wallpaper-failed = Não foi possível trocar o papel de parede ({ $reason }). A imagem está em { $path }
saved = Salva
exported = Imagem salva
open-failed = Não foi possível abrir: { $reason }
save-failed = Não foi possível salvar: { $reason }

## Área da imagem
viewport = Cena renderizada
