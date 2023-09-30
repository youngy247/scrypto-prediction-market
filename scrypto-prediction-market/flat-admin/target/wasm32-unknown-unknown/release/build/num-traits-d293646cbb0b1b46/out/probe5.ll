; ModuleID = 'probe5.295a42e2d93edad2-cgu.0'
source_filename = "probe5.295a42e2d93edad2-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

; core::f64::<impl f64>::is_subnormal
; Function Attrs: inlinehint nounwind
define internal zeroext i1 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$12is_subnormal17hf827a24d3c7cc4bfE"(double %self) unnamed_addr #0 {
start:
  %_2 = alloca i8, align 1
; call core::f64::<impl f64>::classify
  %0 = call i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$8classify17hb6c790aa797cc639E"(double %self) #2, !range !0
  store i8 %0, ptr %_2, align 1
  %1 = load i8, ptr %_2, align 1, !range !0, !noundef !1
  %_3 = zext i8 %1 to i32
  %2 = icmp eq i32 %_3, 3
  ret i1 %2
}

; probe5::probe
; Function Attrs: nounwind
define hidden void @_ZN6probe55probe17hc6561013887c72e6E() unnamed_addr #1 {
start:
; call core::f64::<impl f64>::is_subnormal
  %_1 = call zeroext i1 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$12is_subnormal17hf827a24d3c7cc4bfE"(double 1.000000e+00) #2
  ret void
}

; core::f64::<impl f64>::classify
; Function Attrs: nounwind
declare dso_local i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$8classify17hb6c790aa797cc639E"(double) unnamed_addr #1

attributes #0 = { inlinehint nounwind "target-cpu"="generic" }
attributes #1 = { nounwind "target-cpu"="generic" }
attributes #2 = { nounwind }

!0 = !{i8 0, i8 5}
!1 = !{}
