/* Owned x86 Windows debug control for the pinned KR diagnostic copy.
 * Trace only connect/shutdown/closesocket/WSAAsyncSelect handles and callers.
 * No packet bodies, credentials, forced returns or source-file modifications.
 * Software breakpoints temporarily replace one byte in the owned process;
 * single-step restores it. Concurrent calls during that step may be missed.
 */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <wchar.h>

typedef struct { DWORD address; BYTE original; const char *name; int active; } Point;
typedef struct { DWORD thread; int point; } Step;
static HANDLE process;
static FILE *output;
static ULONGLONG began;
static Point points[5];
static Step steps[64];
static unsigned calls;

static int read_u32(DWORD address, DWORD *value) {
    SIZE_T count=0;
    return ReadProcessMemory(process,(const void *)(uintptr_t)address,value,4,&count) && count==4;
}
static int byte_at(DWORD address, BYTE value) {
    DWORD old=0, ignored=0; SIZE_T count=0;
    if(!VirtualProtectEx(process,(void *)(uintptr_t)address,1,PAGE_EXECUTE_READWRITE,&old)) return 0;
    int ok=WriteProcessMemory(process,(void *)(uintptr_t)address,&value,1,&count) && count==1;
    VirtualProtectEx(process,(void *)(uintptr_t)address,1,old,&ignored);
    FlushInstructionCache(process,(void *)(uintptr_t)address,1);
    return ok;
}
static int install(int index, DWORD address, const char *name) {
    SIZE_T count=0;
    if(!address || !ReadProcessMemory(process,(const void *)(uintptr_t)address,&points[index].original,1,&count) || count!=1) return 0;
    points[index].address=address; points[index].name=name; points[index].active=1;
    return byte_at(address,0xcc);
}
/* CommandLineToArgvW-compatible quoting; the legacy native token is never logged. */
static int append(wchar_t *dest, size_t *used, const wchar_t *arg) {
    if(*used) dest[(*used)++]=L' ';
    dest[(*used)++]=L'"';
    while(*arg) {
        size_t slash=0;
        while(*arg==L'\\') { slash++;arg++; }
        size_t n=(*arg==L'"' || !*arg)?slash*2:slash;
        if(*used+n+4>=32768) return 0;
        while(n--) dest[(*used)++]=L'\\';
        if(!*arg) break;
        if(*arg==L'"') dest[(*used)++]=L'\\';
        dest[(*used)++]=*arg++;
    }
    dest[(*used)++]=L'"'; dest[*used]=0; return 1;
}
int wmain(int argc, wchar_t **argv) {
    if(argc<4) { fputs("usage: socket-probe <local-log> <native-exe> <native-args...>\n",stderr);return 2; }
    HANDLE file=CreateFileW(argv[1],GENERIC_WRITE,FILE_SHARE_READ,NULL,CREATE_NEW,FILE_ATTRIBUTE_NORMAL,NULL);
    if(file==INVALID_HANDLE_VALUE) return 3;
    /* Keep the CRT stream independent of the raw handle. */
    CloseHandle(file); output=_wfopen(argv[1],L"wb");
    if(!output) return 3;
    setvbuf(output,NULL,_IONBF,0); began=GetTickCount64();
    fprintf(output,"{\"event\":\"probe_start\",\"limits\":\"Owned x86 child; software breakpoints; possible concurrent-step omissions; no native acceptance inferred\"}\n");
    wchar_t *command=calloc(32768,sizeof(wchar_t));size_t used=0;
    if(!command) { fclose(output);return 4; }
    for(int i=2;i<argc;i++) if(!append(command,&used,argv[i])) { SecureZeroMemory(command,32768*sizeof(wchar_t));free(command);fclose(output);return 4; }
    STARTUPINFOW startup={0};PROCESS_INFORMATION child={0};startup.cb=sizeof(startup);
    BOOL created=CreateProcessW(argv[2],command,NULL,NULL,FALSE,DEBUG_ONLY_THIS_PROCESS,NULL,NULL,&startup,&child);
    SecureZeroMemory(command,32768*sizeof(wchar_t));free(command);
    if(!created) { fprintf(output,"{\"event\":\"create_failed\",\"win32\":%lu}\n",GetLastError());fclose(output);return 5; }
    DebugSetProcessKillOnExit(TRUE);process=child.hProcess;
    DWORD result=6;int done=0, initial_break=0;
    while(!done && GetTickCount64()-began<180000) {
        DEBUG_EVENT event={0};if(!WaitForDebugEvent(&event,500)) continue;
        DWORD continuation=DBG_CONTINUE;
        if(event.dwDebugEventCode==CREATE_PROCESS_DEBUG_EVENT) {
            DWORD base=(DWORD)(uintptr_t)event.u.CreateProcessInfo.lpBaseOfImage;
            fprintf(output,"{\"event\":\"child_created\",\"pid\":%lu,\"base\":\"0x%lx\"}\n",event.dwProcessId,base);
            if(event.u.CreateProcessInfo.hFile) CloseHandle(event.u.CreateProcessInfo.hFile);
            if(base!=0x400000 || !install(0,base+0x1e66000,"entry")) done=1;
        } else if(event.dwDebugEventCode==LOAD_DLL_DEBUG_EVENT) {
            if(event.u.LoadDll.hFile) CloseHandle(event.u.LoadDll.hFile);
        } else if(event.dwDebugEventCode==EXCEPTION_DEBUG_EVENT) {
            DWORD code=event.u.Exception.ExceptionRecord.ExceptionCode;
            HANDLE thread=OpenThread(THREAD_GET_CONTEXT|THREAD_SET_CONTEXT,FALSE,event.dwThreadId);
            CONTEXT context={0};context.ContextFlags=CONTEXT_CONTROL|CONTEXT_INTEGER;
            if(!thread || !GetThreadContext(thread,&context)) { if(thread) CloseHandle(thread);done=1; }
            else {
                int which=-1;
                for(int i=0;i<5;i++) if(points[i].active && context.Eip-1==points[i].address) which=i;
                if(code==EXCEPTION_BREAKPOINT && which>=0) {
                    Point *point=&points[which];byte_at(point->address,point->original);context.Eip=point->address;
                    if(which==0) {
                        points[0].active=0;
                        DWORD a=0;const DWORD slots[]={0x16ea778,0x16ea790,0x16ea79c,0x16ea740};
                        const char *names[]={"connect","shutdown","closesocket","WSAAsyncSelect"};
                        for(int i=0;i<4;i++) if(!read_u32(slots[i],&a) || !install(i+1,a,names[i])) done=1;
                        fprintf(output,"{\"event\":\"winsock_probes_installed\"}\n");
                    } else {
                        DWORD caller=0,socket=0,second=0;read_u32(context.Esp,&caller);read_u32(context.Esp+4,&socket);read_u32(context.Esp+8,&second);
                        fprintf(output,"{\"event\":\"winsock_call\",\"elapsed_ms\":%llu,\"api\":\"%s\",\"thread\":%lu,\"socket\":%lu,\"caller\":\"0x%lx\",\"second\":%lu}\n",GetTickCount64()-began,point->name,event.dwThreadId,socket,caller,second);
                        if(++calls>2048) done=1;
                        int slot=-1;for(int i=0;i<64;i++) if(!steps[i].thread) { slot=i;break; }
                        if(slot<0) done=1;else { steps[slot].thread=event.dwThreadId;steps[slot].point=which;context.EFlags|=0x100; }
                    }
                    if(!SetThreadContext(thread,&context)) done=1;
                } else if(code==EXCEPTION_SINGLE_STEP) {
                    int found=0;
                    for(int i=0;i<64;i++) if(steps[i].thread==event.dwThreadId) {
                        found=1;byte_at(points[steps[i].point].address,0xcc);steps[i].thread=0;
                    }
                    if(found) { context.EFlags&=~0x100UL;if(!SetThreadContext(thread,&context)) done=1; }
                    else continuation=DBG_EXCEPTION_NOT_HANDLED;
                } else if(code==EXCEPTION_BREAKPOINT && !initial_break) initial_break=1;
                else { continuation=DBG_EXCEPTION_NOT_HANDLED;fprintf(output,"{\"event\":\"exception\",\"code\":%lu,\"address\":\"0x%lx\",\"first_chance\":%lu}\n",code,(DWORD)(uintptr_t)event.u.Exception.ExceptionRecord.ExceptionAddress,event.u.Exception.dwFirstChance); }
                CloseHandle(thread);
            }
        } else if(event.dwDebugEventCode==EXIT_PROCESS_DEBUG_EVENT) {
            result=event.u.ExitProcess.dwExitCode;done=1;
            fprintf(output,"{\"event\":\"native_exit\",\"code\":%lu}\n",result);
        }
        if(!ContinueDebugEvent(event.dwProcessId,event.dwThreadId,continuation)) done=1;
    }
    if(result==6) { fprintf(output,"{\"event\":\"probe_bound_or_failure\"}\n");TerminateProcess(process,6);WaitForSingleObject(process,5000); }
    CloseHandle(child.hThread);CloseHandle(child.hProcess);fclose(output);return (int)result;
}
