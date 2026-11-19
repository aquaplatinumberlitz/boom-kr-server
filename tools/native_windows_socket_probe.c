/* Owned x86 Windows debug control for the pinned KR diagnostic copy.
 * Trace connect/shutdown/closesocket/WSAAsyncSelect handles and callers, plus
 * entry at the measured candidate LoginStage cleanup function (not a cause claim).
 * No packet bodies, credentials, forced returns or source-file modifications.
 * Default observers do not change dialog state. Explicit KR_M8_WAIT_TIMEOUT_MS
 * permits one finite zero-limit timing control in the owned runtime object;
 * the previous/new values are recorded, and source installation stays intact.
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
#define POINT_COUNT 45
static Point points[POINT_COUNT];
static Step steps[64];
static unsigned calls;
static WORD wait_timeout_control;
static int ui_exit_probe;
static int startup_guard_probe;
static int login_input_probe;
static int login_profile_probe;
static DWORD profile_packet;
static unsigned profile_poll_samples;

static int read_u32(DWORD address, DWORD *value) {
    SIZE_T count=0;
    return ReadProcessMemory(process,(const void *)(uintptr_t)address,value,4,&count) && count==4;
}
/* Scalar dialog state selected from the measured DoModal consumer, not a raw
 * object dump. Offsets/numeric values retain unknown semantics until measured. */
static void dialog_fields(DWORD object) {
    const DWORD offsets[]={0xc0,0xcc,0xd4,0xd8};
    DWORD value=0,table=0,target=0;
    for(unsigned i=0;i<4;i++) if(read_u32(object+offsets[i],&value))
        fprintf(output,",\"dialog_offset_%lx\":%lu",offsets[i],value);
    if(read_u32(object,&table) && read_u32(table+168,&target) && target>=0x400000 && target<0x225e000)
        fprintf(output,",\"dialog_slot168\":\"0x%lx\"",target);
    if(read_u32(object+0x460,&value)) fprintf(output,",\"wait_limit_u16\":%lu",value & 0xffffUL);
    if(read_u32(object+0x464,&value)) fprintf(output,",\"wait_start_u32\":%lu",value);
    if(read_u32(object+0x468,&value)) fprintf(output,",\"wait_interval_u32\":%lu",value);
    if(read_u32(object+0x46c,&value)) fprintf(output,",\"wait_flag_u8\":%lu",value & 0xffUL);
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
static int install_expected(int index, DWORD address, const char *name, const BYTE *expected, SIZE_T size) {
    BYTE actual[8];SIZE_T count=0;
    return size<=sizeof(actual) && ReadProcessMemory(process,(const void *)(uintptr_t)address,actual,size,&count)
        && count==size && !memcmp(actual,expected,size) && install(index,address,name);
}
/* Candidate return sites only, never raw stack values or string contents.
 * A matching call opcode is not a validated unwind; optimized code can omit
 * frames and coincidental stack values can still match these filters.
 */
static void return_candidates(DWORD esp) {
    fputs(",\"return_candidates\":[",output);unsigned count=0;
    for(unsigned i=0;i<64 && count<12;i++) {
        DWORD address=0;BYTE code[6];SIZE_T got=0;
        if(!read_u32(esp+i*4,&address) || address<0x401006 || address>=0x225e000) continue;
        if(!ReadProcessMemory(process,(const void *)(uintptr_t)(address-6),code,6,&got) || got!=6) continue;
        if(code[1]!=0xe8 && !(code[0]==0xff && code[1]==0x15)) continue;
        fprintf(output,"%s{\"stack_offset\":%u,\"return\":\"0x%lx\"}",count?",":"",i*4,address);count++;
    }
    fputc(']',output);
}
/* CommandLineToArgvW-compatible quoting; the legacy native token is never logged. */
static int append(wchar_t *dest, size_t *used, const wchar_t *arg) {
    /* Match ordinary CreateProcess argument spelling for simple legacy KR
     * switches. The game may inspect GetCommandLine instead of a CRT argv. */
    if(*arg && !wcspbrk(arg,L" \t\"")) {
        size_t length=wcslen(arg);
        if(*used+length+2>=32768) return 0;
        if(*used) dest[(*used)++]=L' ';
        memcpy(dest+*used,arg,length*sizeof(wchar_t));*used+=length;dest[*used]=0;
        return 1;
    }
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
    if(argc<3) { fputs("usage: socket-probe <local-log> <native-exe> [native-args...]\n",stderr);return 2; }
    HANDLE file=CreateFileW(argv[1],GENERIC_WRITE,FILE_SHARE_READ,NULL,CREATE_NEW,FILE_ATTRIBUTE_NORMAL,NULL);
    if(file==INVALID_HANDLE_VALUE) return 3;
    /* Keep the CRT stream independent of the raw handle. */
    CloseHandle(file); output=_wfopen(argv[1],L"wb");
    if(!output) return 3;
    setvbuf(output,NULL,_IONBF,0); began=GetTickCount64();
    wchar_t timing[16]={0};DWORD timing_size=GetEnvironmentVariableW(L"KR_M8_WAIT_TIMEOUT_MS",timing,16);
    if(timing_size) {
        if(timing_size>=16) { fclose(output);return 2; }
        wchar_t *end=NULL;unsigned long value=wcstoul(timing,&end,10);
        if(!end || *end || !value || value>60000) { fclose(output);return 2; }
        wait_timeout_control=(WORD)value;
    }
    wchar_t ui_probe[4]={0};DWORD ui_size=GetEnvironmentVariableW(L"KR_M8_UI_EXIT_PROBE",ui_probe,4);
    if(ui_size) {
        if(ui_size!=1 || ui_probe[0]!=L'1') { fclose(output);return 2; }
        ui_exit_probe=1;
    }
    wchar_t guard_probe[4]={0};DWORD guard_size=GetEnvironmentVariableW(L"KR_M8_STARTUP_GUARD_PROBE",guard_probe,4);
    if(guard_size) {
        if(guard_size!=1 || guard_probe[0]!=L'1' || ui_exit_probe || wait_timeout_control) { fclose(output);return 2; }
        startup_guard_probe=1;
    }
    wchar_t input_probe[4]={0};DWORD input_size=GetEnvironmentVariableW(L"KR_M8_LOGIN_INPUT_PROBE",input_probe,4);
    if(input_size) {
        if(input_size!=1 || input_probe[0]!=L'1' || ui_exit_probe || startup_guard_probe || wait_timeout_control) { fclose(output);return 2; }
        login_input_probe=1;
    }
    wchar_t profile_probe[4]={0};DWORD profile_size=GetEnvironmentVariableW(L"KR_M8_LOGIN_PROFILE_PROBE",profile_probe,4);
    if(profile_size) {
        if(profile_size!=1 || profile_probe[0]!=L'1' || !login_input_probe) { fclose(output);return 2; }
        login_profile_probe=1;
    }
    fprintf(output,"{\"event\":\"probe_start\",\"wait_timeout_control_ms\":%u,\"limits\":\"Owned x86 child; software breakpoints and selected UI-thread debug-register observation; explicit zero-only timing control if nonzero; possible concurrent-step omissions; no native acceptance inferred\"}\n",(unsigned)wait_timeout_control);
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
            CONTEXT context={0};context.ContextFlags=CONTEXT_CONTROL|CONTEXT_INTEGER|CONTEXT_DEBUG_REGISTERS;
            if(!thread || !GetThreadContext(thread,&context)) { if(thread) CloseHandle(thread);done=1; }
            else {
                int which=-1;
                for(int i=0;i<POINT_COUNT;i++) if(points[i].active && context.Eip-1==points[i].address) which=i;
                if(code==EXCEPTION_BREAKPOINT && which>=0) {
                    Point *point=&points[which];byte_at(point->address,point->original);context.Eip=point->address;
                    if(which==0) {
                        points[0].active=0;
                        if(login_input_probe) {
                            if(!install_expected(21,0x1101ba0,"manual_credentials",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\x08\x21",8)
                               || !install_expected(22,0x11041a0,"trial_empty_credentials",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\x4d\x23",8)
                               || !install_expected(23,0x1101b70,"roster_select",(const BYTE *)"\x55\x8b\xec\x51\x89\x4d\xfc\x8b",8)
                               || !install_expected(24,0x1101690,"controls_enable",(const BYTE *)"\x55\x8b\xec\x83\xec\x0c\x89\x4d",8)
                               || !install_expected(25,0x1101e20,"manual_confirm",(const BYTE *)"\x55\x8b\xec\x83\xec\x08\x89\x4d",8)
                               || !install_expected(26,0x1104090,"trial_confirm",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\x2c\x23",8)
                               || !install_expected(27,0x13adf3b,"builder_web_selection",(const BYTE *)"\x85\xc0",2)
                               || !install_expected(28,0x13ae224,"record_web_selection",(const BYTE *)"\x85\xc0",2)
                               || !install_expected(29,0x13ae24a,"manual_record_writer",(const BYTE *)"\xe8\x91\xe5\x1a\xff\x89\x45\x94",8)
                               || !install_expected(30,0x13ae109,"invalid_login_notice",(const BYTE *)"\x68\x84\x8f\x78\x01\x6a\x00\xff",8)) done=1;
                            fprintf(output,"{\"event\":\"login_input_probes_installed\",\"result_mutation\":false,\"secret_reads\":false}\n");
                            if(login_profile_probe) {
                                if(!install_expected(31,0x10fd540,"auth_profile_entry",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\x46\x1c",8)
                                   || !install_expected(32,0x10fed80,"auth_profile_return",(const BYTE *)"\x8b\x4d\xf4\x64\x89\x0d\x00\x00",8)
                                   || !install_expected(33,0x13b09e0,"auth_local_id_set",(const BYTE *)"\x55\x8b\xec\x51\x89\x4d\xfc\x8b",8)
                                   || !install_expected(34,0x1100590,"netinfo_profile_entry",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\xa9\x1e",8)
                                   || !install_expected(7,0x1107412,"login_wait_return",(const BYTE *)"\x89\x45\xdc\x83\x7d\xdc\x02\x75",8)
                                   || !install_expected(35,0x1104410,"manual_wait_entry",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\x7c\x23",8)
                                   || !install_expected(36,0x11044a7,"manual_wait_return",(const BYTE *)"\x89\x45\xec\x8b\x4d\xf0\xc7\x81",8)
                                   || !install_expected(37,0x1103bb0,"after_login_entry",(const BYTE *)"\x55\x8b\xec\x83\xec\x1c\x89\x4d",8)
                                   || !install_expected(38,0x1103c47,"after_login_feature",(const BYTE *)"\x0f\xb6\xd0\x85\xd2\x74\x0b\x8b",8)
                                   || !install_expected(39,0x1107b27,"special_init_return",(const BYTE *)"\x0f\xb6\xc0\x85\xc0\x75\x07\x32",8)
                                   || !install_expected(40,0x1107c7e,"special_init_success",(const BYTE *)"\xb0\x01\x8b\x4d\xf4\x64\x89\x0d",8)
                                   || !install_expected(42,0xd88280,"flags62_entry",(const BYTE *)"\x55\x8b\xec\x83\xec\x0c\x89\x4d",8)
                                   || !install_expected(43,0x13b27e0,"flags62_set",(const BYTE *)"\x55\x8b\xec\x51\x89\x4d\xfc\x0f",8)
                                   || !install_expected(44,0x1107c90,"channel_request_entry",(const BYTE *)"\x55\x8b\xec\x6a\xff\x68\x56\x26",8)) done=1;
                            }
                        } else if(startup_guard_probe) {
                            /* Exact current vi/ko WinMain bytes are pinned by
                             * the caller. Observe returns and rejection entry;
                             * never replace results or bypass the mutex. */
                            if(!install_expected(17,0x1253cfd,"startup_mutex_return",(const BYTE *)"\x89\x45\xe4",3)
                               || !install_expected(18,0x1253d0c,"startup_last_error_return",(const BYTE *)"\x3d\xb7\x00\x00\x00",5)
                               || !install_expected(19,0x1253d22,"startup_wait_return",(const BYTE *)"\x3d\x02\x01\x00\x00",5)
                               || !install_expected(20,0x1253d29,"startup_multi_execute_rejected",(const BYTE *)"\x68\x3c\x21\x79\x01",5)) done=1;
                            fprintf(output,"{\"event\":\"startup_guard_probes_installed\",\"result_mutation\":false}\n");
                        } else {
                        DWORD a=0;const DWORD slots[]={0x16ea778,0x16ea790,0x16ea79c,0x16ea740};
                        const char *names[]={"connect","shutdown","closesocket","WSAAsyncSelect"};
                        for(int i=0;i<4;i++) if(!read_u32(slots[i],&a) || !install(i+1,a,names[i])) done=1;
                        if(!install(5,0x1107a10,"login_stage_close_candidate")) done=1;
                        /* A static wait trampoline occupies the whole six-byte
                         * call site. Never plant a breakpoint in its rel32.
                         * The pinned executable and copied probe source remain
                         * the authority for each declared run. */
                        BYTE wait_call[6]={0};SIZE_T wait_read=0;
                        if(!ReadProcessMemory(process,(void *)(uintptr_t)0x110740c,wait_call,6,&wait_read) || wait_read!=6) done=1;
                        else if(!memcmp(wait_call,"\x8b\x4d\xe4\xff\x55\xe0",6)) {
                            if(!install(6,0x110740f,"login_dialog_call")) done=1;
                        } else if(wait_call[0]==0xe9 && wait_call[5]==0x90 && !wait_timeout_control) {
                            fprintf(output,"{\"event\":\"static_wait_hook_observed\",\"call_probe_disabled\":true}\n");
                        } else done=1;
                        if(!install(7,0x1107412,"login_dialog_return")) done=1;
                        if(ui_exit_probe) {
                            if(!install(10,0xc1e670,"exit_button_handler")
                               || !install(11,0xbeaa0d,"exit_modal_return")
                               || !install(12,0xbeab37,"exit_predicate_return")
                               || !install(13,0xbeac36,"exit_prepare_return")
                               || !install(14,0xbeac4e,"exit_optional_prepare_return")
                               || !install(16,0xbeac7a,"exit_virtual328_return")) done=1;
                        }
                        fprintf(output,"{\"event\":\"winsock_probes_installed\"}\n");
                        }
                    } else {
                        DWORD caller=0,socket=0,second=0;read_u32(context.Esp,&caller);read_u32(context.Esp+4,&socket);read_u32(context.Esp+8,&second);
                        if(which>=5) {
                            fprintf(output,"{\"event\":\"%s\",\"elapsed_ms\":%llu,\"function\":\"%s\",\"thread\":%lu,\"point\":\"0x%lx\"",(which==5 || which==8)?"native_candidate_entry":"native_point",GetTickCount64()-began,point->name,event.dwThreadId,point->address);
                            if(which==5 || which==8) fprintf(output,",\"caller\":\"0x%lx\"",caller);
                            if(which==8) fprintf(output,",\"argument1\":%lu,\"argument2\":%lu",socket,second);
                            if(which==10) {
                                DWORD table=0;
                                fprintf(output,",\"button_id\":%lu",socket);
                                if(context.Ecx>=172 && read_u32(context.Ecx-172,&table))
                                    fprintf(output,",\"dialog_vtable\":\"0x%lx\"",table);
                            }
                            if(which>=17 && which<=20) fprintf(output,",\"eax\":%lu",context.Eax);
                            if(which>=21 && which<=26) {
                                DWORD selected=0;
                                if(context.Ecx>=0x2000000 && context.Ecx<0x70000000
                                   && read_u32(context.Ecx+0x1270,&selected) && selected<=2)
                                    fprintf(output,",\"roster_field_1270\":%lu",selected);
                                DWORD automatic=0;
                                if(context.Ecx>=0x2000000 && context.Ecx<0x70000000
                                   && read_u32(context.Ecx+0x2fe4,&automatic) && automatic<=1)
                                    fprintf(output,",\"automatic_input_lock_2fe4\":%lu",automatic);
                                if((which==23 || which==24) && socket<=2)
                                    fprintf(output,",\"argument1\":%lu",socket);
                            }
                            if(which>=27 && which<=30) {
                                DWORD count=0;
                                if(read_u32(context.Ebp-0x24,&count) && count<=2)
                                    fprintf(output,",\"credential_count\":%lu",count);
                                if((which==27 || which==28) && context.Eax<=1)
                                    fprintf(output,",\"web_selection\":%lu",context.Eax);
                            }
                            if(which==31) read_u32(context.Esp+4,&profile_packet);
                            if(which==31 || which==32) {
                                DWORD cursor=0,size=0,error54=0,error58=0;
                                if(profile_packet && read_u32(profile_packet+0x48,&cursor)
                                   && read_u32(profile_packet+0x44,&size)
                                   && read_u32(profile_packet+0x54,&error54)
                                   && read_u32(profile_packet+0x58,&error58))
                                    fprintf(output,",\"cursor\":%lu,\"payload_bytes\":%lu,\"error54\":%lu,\"error58\":%lu",cursor,size,error54,error58);
                                if(which==32) profile_packet=0;
                            }
                            if(which==33) {
                                DWORD slot=0,identity=0;
                                if(read_u32(context.Esp+4,&slot) && slot<=1
                                   && read_u32(context.Esp+8,&identity) && identity<=0x7fffffffUL)
                                    fprintf(output,",\"local_slot\":%lu,\"wire_identity\":%lu",slot,identity);
                            }
                            if(which==34 || which==35 || which==36) {
                                DWORD stage=context.Ecx,waiting=0,table=0,target=0;
                                if(which==36) {
                                    read_u32(context.Ebp-0x10,&stage);
                                    fprintf(output,",\"manual_wait_result\":%lu",context.Eax);
                                }
                                if(stage>=0x2000000 && stage<0x70000000) {
                                    if(read_u32(stage+0x2fd8,&waiting) && waiting<=1)
                                        fprintf(output,",\"wait_owned_2fd8\":%lu",waiting);
                                    if(read_u32(stage+0x2c04,&table) && read_u32(table+152,&target)
                                       && target>=0x400000 && target<0x2300000)
                                        fprintf(output,",\"manual_wait_target\":\"0x%lx\"",target);
                                }
                            }
                            if(which==38 || which==39)
                                fprintf(output,",\"result_u8\":%lu",context.Eax & 255UL);
                            if(which==40) {
                                /* Install only after successful stage setup. The
                                 * first sixteen samples bound debugger overhead. */
                                if(!install_expected(41,0x11076a9,"handoff_predicate_return",
                                    (const BYTE *)"\x83\xc4\x08\x0f\xb6\xc8\x85\xc9",8)) done=1;
                            }
                            if(which==41) {
                                DWORD a=0,b=0,slot=0;
                                if(read_u32(context.Ebp-0x20,&a) && read_u32(context.Ebp-0x1c,&b))
                                    fprintf(output,",\"predicate_a\":%lu,\"predicate_b\":%lu,\"result_u8\":%lu",a,b,context.Eax & 255UL);
                                if(read_u32(context.Ebp-8,&slot) && slot<=1)
                                    fprintf(output,",\"local_slot\":%lu",slot);
                                if(++profile_poll_samples>=16) point->active=0;
                            }
                            if(which==43 && socket<=1)
                                fprintf(output,",\"local_slot\":%lu,\"status\":%lu",socket,second);
                            if(which==44)
                                fprintf(output,",\"channel_argument\":%lu",socket);
                            if(which>=11 && which<=16) {
                                DWORD object=0,flags=0,table=0,target=0;
                                fprintf(output,",\"eax\":%lu",context.Eax);
                                if(read_u32(context.Ebp-0xe,&flags))
                                    fprintf(output,",\"relogin_byte\":%lu,\"admitted_byte\":%lu",flags & 255UL,(flags>>8)&255UL);
                                /* BEA8A0's v35 is EBP-30h; -34h is its
                                 * preceding scalar, not the stage pointer. */
                                if(read_u32(context.Ebp-0x30,&object) && read_u32(object,&table)) {
                                    if(read_u32(table+328,&target) && target>=0x400000 && target<0x2300000) {
                                        fprintf(output,",\"exit_slot328\":\"0x%lx\"",target);
                                        if(which==12 && !points[15].active
                                           && !install(15,target,"exit_virtual328_entry")) {
                                            fputs(",\"dynamic_exit_probe_failed\":true",output);done=1;
                                        }
                                    }
                                    if(read_u32(table+332,&target) && target>=0x400000 && target<0x2300000)
                                        fprintf(output,",\"relogin_slot332\":\"0x%lx\"",target);
                                }
                                if(which==15) {
                                    fprintf(output,",\"caller\":\"0x%lx\"",caller);
                                    if(caller==0xbeac7a) points[15].active=0;
                                }
                            }
                            if(which==9) {
                                DWORD object=0;
                                fprintf(output,",\"native_now_u32\":%lu",context.Eax);
                                if(read_u32(context.Ebp-4,&object)) dialog_fields(object);
                            }
                            if(which==6) {
                                DWORD target=0;
                                if(read_u32(context.Ebp-0x20,&target) && target>=0x400000 && target<0x225e000)
                                    fprintf(output,",\"dialog_target\":\"0x%lx\"",target);
                                dialog_fields(context.Ecx);
                                if(!points[8].active && !install(8,0xd66b80,"stage_notify_candidate")) done=1;
                                if(!points[9].active && !install(9,0x11139ff,"wait_timer_now")) done=1;
                                if(wait_timeout_control) {
                                    DWORD old=0;SIZE_T changed=0;
                                    if(!read_u32(context.Ecx+0x460,&old) || (old & 0xffffUL)!=0
                                       || !WriteProcessMemory(process,(void *)(uintptr_t)(context.Ecx+0x460),&wait_timeout_control,2,&changed) || changed!=2) {
                                        fputs(",\"wait_timing_control_failed\":true",output);done=1;
                                    } else {
                                        fprintf(output,",\"wait_timing_control_ms\":%u,\"previous_limit\":0",(unsigned)wait_timeout_control);
                                    }
                                }
                                /* Observe writes to the measured numeric result
                                 * on this owned UI thread only; no value is forced. */
                                context.Dr0=context.Ecx+0xc0;
                                context.Dr6=0;
                                context.Dr7=(context.Dr7 & ~0xf0003UL) | 0xd0001UL;
                            }
                            if(which==7) {
                                DWORD object=0;
                                fprintf(output,",\"dialog_result\":%lu",context.Eax);
                                if(read_u32(context.Ebp-0x1c,&object)) dialog_fields(object);
                                context.Dr0=0;
                                context.Dr6=0;
                                context.Dr7&=~0xf0003UL;
                                for(int index=8;index<10;index++) if(points[index].active) {
                                    if(!byte_at(points[index].address,points[index].original)) done=1;
                                    points[index].active=0;
                                }
                            }
                        } else {
                            fprintf(output,"{\"event\":\"winsock_call\",\"elapsed_ms\":%llu,\"api\":\"%s\",\"thread\":%lu,\"socket\":%lu,\"caller\":\"0x%lx\",\"second\":%lu",GetTickCount64()-began,point->name,event.dwThreadId,socket,caller,second);
                        }
                        return_candidates(context.Esp);
                        DWORD state=0,error=0;
                        if(read_u32(0x1a966fc,&state) && read_u32(0x1a3398c,&error))
                            fprintf(output,",\"bootstrap_flag\":%lu,\"error_word\":%lu",state,error);
                        fputs("}\n",output);
                        if(++calls>2048) done=1;
                        int slot=-1;for(int i=0;i<64;i++) if(!steps[i].thread) { slot=i;break; }
                        if(slot<0) done=1;else { steps[slot].thread=event.dwThreadId;steps[slot].point=which;context.EFlags|=0x100; }
                    }
                    if(!SetThreadContext(thread,&context)) done=1;
                } else if(code==EXCEPTION_SINGLE_STEP) {
                    int found=0;
                    if(context.Dr6 & 1) {
                        DWORD value=0;found=1;
                        fprintf(output,"{\"event\":\"dialog_result_write\",\"elapsed_ms\":%llu,\"thread\":%lu,\"instruction_after_write\":\"0x%lx\"",GetTickCount64()-began,event.dwThreadId,context.Eip);
                        if(read_u32(context.Dr0,&value)) fprintf(output,",\"value\":%lu",value);
                        if(context.Eip==0x1503c8e && read_u32(context.Ebp+4,&value) && value>=0x400000 && value<0x225e000)
                            fprintf(output,",\"setter_frame_return\":\"0x%lx\"",value);
                        return_candidates(context.Esp);fputs("}\n",output);
                        context.Dr6=0;
                    }
                    for(int i=0;i<64;i++) if(steps[i].thread==event.dwThreadId) {
                        found=1;
                        if(points[steps[i].point].active) byte_at(points[steps[i].point].address,0xcc);
                        steps[i].thread=0;
                    }
                    if(found) { context.EFlags&=~0x100UL;if(!SetThreadContext(thread,&context)) done=1; }
                    else continuation=DBG_EXCEPTION_NOT_HANDLED;
                } else if(code==EXCEPTION_BREAKPOINT && !initial_break) initial_break=1;
                else {
                    continuation=DBG_EXCEPTION_NOT_HANDLED;
                    fprintf(output,"{\"event\":\"exception\",\"code\":%lu,\"address\":\"0x%lx\",\"first_chance\":%lu,\"eip\":\"0x%lx\",\"ebp\":\"0x%lx\",\"esp\":\"0x%lx\"",code,(DWORD)(uintptr_t)event.u.Exception.ExceptionRecord.ExceptionAddress,event.u.Exception.dwFirstChance,context.Eip,context.Ebp,context.Esp);
                    return_candidates(context.Esp);fputs("}\n",output);
                }
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
