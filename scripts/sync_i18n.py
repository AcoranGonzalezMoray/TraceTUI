import os
import json

def actualizar_y_ordenar_jsons(ruta_json_principal, ruta_carpeta_destino):
    try:
        with open(ruta_json_principal, 'r', encoding='utf-8') as f:
            datos_principal = json.load(f)
            if not isinstance(datos_principal, dict):
                print("Error: El JSON principal debe ser un objeto/diccionario.")
                return
            # Guardamos las claves del JSON principal
            claves_principales = datos_principal.keys()
    except Exception as e:
        print(f"Error al leer el JSON principal: {e}")
        return

    print(f"Claves detectadas en el JSON principal: {list(claves_principales)}\n")

    if not os.path.exists(ruta_carpeta_destino):
        print(f"La ruta de la carpeta no existe: {ruta_carpeta_destino}")
        return

    for archivo in os.listdir(ruta_carpeta_destino):
        if archivo.endswith('.json'):
            ruta_archivo_completa = os.path.join(ruta_carpeta_destino, archivo)
            
            try:
                with open(ruta_archivo_completa, 'r', encoding='utf-8') as f:
                    datos_secundario = json.load(f)
                
                if isinstance(datos_secundario, dict):
                    for clave in claves_principales:
                        if clave not in datos_secundario:
                            datos_secundario[clave] = ""
                    
                    with open(ruta_archivo_completa, 'w', encoding='utf-8') as f:
                        json.dump(datos_secundario, f, ensure_ascii=False, indent=4, sort_keys=True)
                    
                    print(f"✔ Procesado y ordenado alfabéticamente: {archivo}")
                else:
                    print(f"⚠ Saltado {archivo}: No es un diccionario válido.")
                    
            except Exception as e:
                print(f"❌ Error al procesar el archivo {archivo}: {e}")

ruta_principal = "../src/i18n/en.json"
carpeta_jsons = "../src/i18n"

actualizar_y_ordenar_jsons(ruta_principal, carpeta_jsons)