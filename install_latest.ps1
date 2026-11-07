# 1. Скачиваем артефакты последнего успешного запуска воркфлоу
Write-Host "Скачивание артефактов с GitHub..." -ForegroundColor Cyan
gh run download --name android

# 2. Ищем скачанный apk-файл в текущей папке или подпапках
$apkFile = Get-ChildItem -Recurse -Filter "*.apk" | Select-Object -First 1

if ($null -eq $apkFile) {
    Write-Error "APK-файл не найден среди скачанных артефактов!"
    exit 1
}

Write-Host "Найден файл: $($apkFile.FullName)" -ForegroundColor Green

# 3. Устанавливаем приложение на телефон через ADB
Write-Host "Установка приложения на устройство..." -ForegroundColor Cyan
adb install -r $apkFile.FullName

if ($LASTEXITCODE -eq 0) {
    Write-Host "Приложение успешно установлено!" -ForegroundColor Green
} else {
    Write-Error "Ошибка при установке через adb."
}
